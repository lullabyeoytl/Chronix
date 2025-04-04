use core::{ops::DerefMut, time::Duration};

use alloc::{boxed::Box, vec};
use listen_table::ListenTable;
use log::info;
use smoltcp::{iface::{Config, Interface, SocketHandle, SocketSet,PollResult}, phy::Medium, socket::{tcp::{Socket, SocketBuffer}, AnySocket}, time::Instant, wire::{EthernetAddress, HardwareAddress}};
use spin::{Lazy, Once};

use crate::{devices::{net::NetDeviceWrapper, NetDevice}, sync::{mutex::{SpinNoIrq, SpinNoIrqLock}, UPSafeCell}, timer::{get_current_time_duration, get_current_time_us, timer::{Timer, TimerEvent, TIMER_MANAGER}}};
#[allow(dead_code)]
/// Network Address Module
pub mod addr;
#[allow(dead_code)]
/// Network Socket Module
pub mod socket;
#[allow(dead_code)]
/// TCP Module
pub mod tcp;
#[allow(dead_code)]
/// A Listen Table for Server to allocte port
pub mod listen_table;
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
/// socket address family
pub enum SaFamily {
    /// ipv4
    AfInet = 2,
    /// ipv6
    AfInet6 = 10,
}

impl TryFrom<u16> for SaFamily {
    type Error = crate::syscall::sys_error::SysError;
    fn try_from(value: u16) -> Result<Self,Self::Error> {
        match value {
            2 => Ok(Self::AfInet),
            10 => Ok(Self::AfInet6),
            _ => Err(Self::Error::EINVAL),
        }
    }
}

const SOCK_RAND_SEED: u64 = 404;// for random port allocation
const CONFIG_RANDOM_SEED: u64 = 0x3A0C_1495_BC68_9A2C; // for smoltcp random seed
const PORT_START: u16 = 0xc000; // 49152
const PORT_END: u16 = 0xffff;   // 65535

const LISTEN_QUEUE_SIZE: usize = 512;
static LISTEN_TABLE: Lazy<ListenTable> = Lazy::new(ListenTable::new);

/// A wrapper for SocketSet in smoltcp
struct SocketSetWrapper<'a>(SpinNoIrqLock<SocketSet<'a>>) ; 
static SOCKET_SET: Lazy<SocketSetWrapper> = Lazy::new(SocketSetWrapper::new);

/// TCP RX and TX buffer size
pub const TCP_RX_BUF_LEN: usize = 64 * 1024;
/// TCP RX and TX buffer size
pub const TCP_TX_BUF_LEN: usize = 64 * 1024;

static ETH0: Once<InterfaceWrapper> = Once::new();
/// A wrapper for interface in smoltcp
struct InterfaceWrapper {
    /// The name of the network interface.
    name: &'static str,
    /// The Ethernet address of the network interface.
    ether_addr: EthernetAddress,
    /// The device wrapper protected by a `Mutex` to ensure thread-safe access.
    dev: SpinNoIrqLock<NetDeviceWrapper>,
    /// The network interface protected by a `Mutex` to ensure thread-safe
    /// access.
    iface: SpinNoIrqLock<Interface>,
}

impl InterfaceWrapper {
    fn new(name: &'static str, dev: Box<dyn NetDevice>, ether_addr: EthernetAddress) -> Self {
        let mut config = match dev.capabilities().medium {
            Medium::Ethernet => Config::new(HardwareAddress::Ethernet(ether_addr)),
            Medium::Ip => Config::new(HardwareAddress::Ip),
        };
        config.random_seed = CONFIG_RANDOM_SEED;
        let mut raw_dev = NetDeviceWrapper::new(dev);
        let iface = SpinNoIrqLock::new(Interface::new(config, &mut raw_dev, Self::current_time()));
        Self {
            name,
            ether_addr,
            dev:SpinNoIrqLock::new(raw_dev),
            iface,
        }
    }

    fn current_time() -> Instant {
        Instant::from_micros_const(get_current_time_us() as i64)
    }
    /// poll the interface 
    pub fn poll(&self, sockets: &SpinNoIrqLock<SocketSet>) -> Instant {
        let mut dev =  self.dev.lock();
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        let timestamp = Self::current_time();
        iface.poll(timestamp, dev.deref_mut(), &mut sockets);
        timestamp
    }

    pub fn check_poll(&self, timestamp: Instant, sockets: &SpinNoIrqLock<SocketSet>) {
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        match iface.poll_delay(timestamp, &mut sockets)
        .map(smol_dur_to_core_cur){
            Some(Duration::ZERO) => {
                iface.poll(Self::current_time(), self.dev.lock().deref_mut(), &mut sockets);
            }
            Some(delay) => {
                // current time + delay is the deadline for the next poll
                let next_poll_deadline = delay +  Duration::from_micros(timestamp.micros() as u64);
                let current_time = get_current_time_duration();
                if next_poll_deadline < current_time {
                    iface.poll(Self::current_time(), self.dev.lock().deref_mut(), &mut sockets);
                }else {
                    let timer = Timer::new(next_poll_deadline, Box::new(NetPollTimer{}));
                    TIMER_MANAGER.add_timer(timer);
                }
            }
            // when return None means no active sockets or all the sockets are handled
            None => {
                // do nothing, just call poll interface
                let empty_timer = Timer::new(get_current_time_duration()+Duration::from_millis(5), Box::new(NetPollTimer{}));
                TIMER_MANAGER.add_timer(empty_timer);
            }
        }

    }

}
impl <'a> SocketSetWrapper<'a> {
    fn new() -> Self {
        let socket_set = SocketSet::new(vec![]);
        Self(SpinNoIrqLock::new(socket_set))
    }
    /// allocate tx buffer and rx buffer ,return a Socket struct in smoltcp
    pub fn new_tcp_socket() -> Socket<'a> {
        let rx_buffer = SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tx_buffer = SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        Socket::new(rx_buffer, tx_buffer)
    }
    /// add a socket to the set , return a socket_handle
    pub fn add_socket<T:AnySocket<'a>>(&self, socket: T) -> SocketHandle {
        let handle = self.0.lock().add(socket);
        info!("[SocketSetWrapper] add_socket handle {:?}" , handle);
        handle
    }
    /// use a ref of socket and do something with it
    pub fn with_socket<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let set = self.0.lock();
        let socket = set.get(handle);
        f(socket)
    }
    /// use a mut ref of socket and do something with it
    pub fn with_socket_mut<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut set = self.0.lock();
        let socket = set.get_mut(handle);
        f(socket)
    }
    pub fn poll_interfaces(&self) -> Instant {
        ETH0.get()
        .unwrap()
        .poll(&self.0)
    }

    pub fn check_poll(&self, timestamp: Instant) {
        ETH0.get()
        .unwrap()
        .check_poll(timestamp, &self.0)
    }

    pub fn remove(&self, handle: SocketHandle) {
        self.0.lock().remove(handle);
        info!("socket {:?}: destroyed", handle);
    }
}


// function or struct concerning time ,from microseconds to smoltcp::time::Instant, from core::time::Duration to smoltcp::time::Duration
/// from core::time::Duration to smoltcp::time::Duration
struct NetPollTimer;
impl TimerEvent for NetPollTimer {
    fn callback(self: Box<Self>) -> Option<Timer> {
        SOCKET_SET.poll_interfaces();
        None
    }
}
pub fn smol_dur_to_core_cur(duration: smoltcp::time::Duration) -> core::time::Duration {
    core::time::Duration::from_micros(duration.micros())
}