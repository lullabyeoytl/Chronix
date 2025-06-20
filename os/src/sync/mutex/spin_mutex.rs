use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
};

use crate::utils::async_utils::SendWrapper;
use super::MutexSupport;

/// A spin-lock based mutex.
pub struct MutexGuard<'a, T: ?Sized, S: MutexSupport> {
    mutex: &'a SpinMutex<T, S>,
    support_guard: S::GuardData,
}

/// `SpinMutex` can include different `MutexSupport` type
pub struct SpinMutex<T: ?Sized, S: MutexSupport> {
    lock: AtomicBool,
    _marker: PhantomData<S>,
    data: UnsafeCell<T>,
}

// Forbid Mutex step over `await` and lead to dead lock
impl<'a, T: ?Sized, S: MutexSupport> !Sync for MutexGuard<'a, T, S> {}
impl<'a, T: ?Sized, S: MutexSupport> !Send for MutexGuard<'a, T, S> {}

unsafe impl<T: ?Sized + Send, S: MutexSupport> Sync for SpinMutex<T, S> {}
unsafe impl<T: ?Sized + Send, S: MutexSupport> Send for SpinMutex<T, S> {}

impl<T, S: MutexSupport> SpinMutex<T, S> {
    /// Construct a SpinMutex
    pub const fn new(user_data: T) -> Self {
        SpinMutex {
            lock: AtomicBool::new(false),
            _marker: PhantomData,
            data: UnsafeCell::new(user_data),
        }
    }

    /// Wait until the lock looks unlocked before retrying
    #[inline(always)]
    fn wait_unlock(&self) {
        let mut try_count = 0usize;
        while self.lock.load(Ordering::Acquire) {
            core::hint::spin_loop();
            try_count += 1;
            if try_count == 0x1000000 {
                panic!("Mutex: deadlock detected! try_count > {:#x}\n", try_count);
            }
        }
    }

    /// Note that the locked data cannot step over `await`,
    /// i.e. cannot be sent between thread.
    #[inline(always)]
    pub fn lock(&self) -> MutexGuard<T, S> {
        loop {
            self.wait_unlock();
            let support_guard = S::before_lock();
            if self
                .lock
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return MutexGuard {
                    mutex: self,
                    support_guard,
                }
            }
        }
    }

    /// # Safety
    ///
    /// This is highly unsafe.
    /// You should ensure that context switch won't happen during
    /// the locked data's lifetime.
    #[inline(always)]
    pub unsafe fn sent_lock(&self) -> impl DerefMut<Target = T> + '_ {
        SendWrapper::new(self.lock())
    }

    #[inline(always)]
    /// get the inner data
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Deref for MutexGuard<'a, T, S> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> DerefMut for MutexGuard<'a, T, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Drop for MutexGuard<'a, T, S> {
    /// The dropping of the MutexGuard will release the lock it was created
    /// from.
    #[inline(always)]
    fn drop(&mut self) {
        self.mutex.lock.store(false, Ordering::Release);
        S::after_unlock(&mut self.support_guard);
    }
}
