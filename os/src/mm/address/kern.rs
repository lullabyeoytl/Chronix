
use core::{fmt::{self, Debug, Formatter}, iter::Step, ops::{Add, AddAssign, Range, Sub, SubAssign}, ptr::{slice_from_raw_parts_mut, NonNull}};

use crate::{config::{KERNEL_ADDR_OFFSET, PAGE_SIZE, PAGE_SIZE_BITS}, mm::PageTableEntry};

use super::{PhysAddr, PhysPageNum, VA_WIDTH_SV39, VPN_WIDTH_SV39};

/// kernel virtual address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct KernAddr(pub usize);

/// kernel page number
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct KernPageNum(pub usize);

#[allow(missing_docs)]
impl KernPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry; 512] {
        self.get_mut()
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8; 4096] {
        self.get_mut()
    }

    ///Get reference to `PhysPageNum` value
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { ((self.0 << PAGE_SIZE_BITS) as *const T).as_ref().unwrap() }
    }
    ///Get mutable reference to `PhysPageNum` value
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { ((self.0 << PAGE_SIZE_BITS) as *mut T).as_mut().unwrap() }
    }
    
    pub fn to_phys(&self) -> PhysPageNum {
        PhysPageNum(self.0 & (KERNEL_ADDR_OFFSET >> 12))
    }
}

#[allow(missing_docs)]
impl KernAddr {
    ///Get reference to `PhysAddr` value
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }
    ///Get mutable reference to `PhysAddr` value
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }

    pub fn as_non_null_ptr<T>(&self) -> NonNull<T> {
        NonNull::new(self.0 as *mut T).unwrap()
    }

    pub fn page_offset(&self) -> usize {
        self.0 & ((1 << PAGE_SIZE_BITS) - 1)
    }

    pub fn floor(&self) -> KernPageNum {
        KernPageNum(self.0 >> PAGE_SIZE_BITS)
    }

    pub fn ceil(&self) -> KernPageNum {
        if self.0 == 0 {
            KernPageNum(0)
        } else {
            KernPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_SIZE_BITS)
        }
    }

    pub fn to_phys(&self) -> PhysAddr {
        PhysAddr(self.0 - KERNEL_ADDR_OFFSET)
    }

}

impl From<KernAddr> for KernPageNum {
    fn from(v: KernAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl From<KernPageNum> for KernAddr {
    fn from(v: KernPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl Debug for KernAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("KA:{:#x}", self.0))
    }
}
impl Debug for KernPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("KPN:{:#x}", self.0))
    }
}

impl From<usize> for KernAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}
impl From<usize> for KernPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl From<KernAddr> for usize {
    fn from(v: KernAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}
impl From<KernPageNum> for usize {
    fn from(v: KernPageNum) -> Self {
        v.0
    }
}

impl Add<usize> for KernAddr {
    type Output = KernAddr;

    fn add(self, rhs: usize) -> Self::Output {
        KernAddr(self.0 + rhs)
    }
}


impl Add<usize> for KernPageNum {
    type Output = KernPageNum;

    fn add(self, rhs: usize) -> Self::Output {
        KernPageNum(self.0 + rhs)
    }
}

impl AddAssign<usize> for KernAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}


impl AddAssign<usize> for KernPageNum {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}


impl Sub<usize> for KernAddr {
    type Output = KernAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        KernAddr(self.0 + rhs)
    }
}


impl Sub<usize> for KernPageNum {
    type Output = KernPageNum;

    fn sub(self, rhs: usize) -> Self::Output {
        KernPageNum(self.0 + rhs)
    }
}

impl SubAssign<usize> for KernAddr {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}


impl SubAssign<usize> for KernPageNum {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}


impl Step for KernAddr {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        usize::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        usize::forward_checked(start.0, count).map(|e| Self(e))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        usize::backward_checked(start.0, count).map(|e| Self(e))
    }
} 


impl Step for KernPageNum {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        usize::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        usize::forward_checked(start.0, count).map(|e| Self(e))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        usize::backward_checked(start.0, count).map(|e| Self(e))
    }
} 

#[allow(missing_docs)]
pub trait RangeKpnData {
    fn get_slice<T>(&self) -> &'static mut [T];
}

impl RangeKpnData for Range<KernPageNum> {
    fn get_slice<T>(&self) -> &'static mut [T] {
        unsafe {
            &mut *slice_from_raw_parts_mut(self.start.get_mut(), (self.clone().count() << PAGE_SIZE_BITS) / size_of::<T>())
        }
    }
}

/// cast to Range<KernPageNum>
pub trait ToRangeKpn {
    /// to kern
    fn to_kern(&self) -> core::ops::Range<KernPageNum>;
}

