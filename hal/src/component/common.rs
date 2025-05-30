use core::ops::Range;

use crate::allocator::{DynamicFrameAllocator, FrameAllocatorHal};

use super::addr::PhysPageNum;

#[derive(Debug)]
pub struct FrameTracker<A: FrameAllocatorHal = DynamicFrameAllocator> {
    pub range_ppn: Range<PhysPageNum>,
    alloc: A
}

impl<A: FrameAllocatorHal> FrameTracker<A> {
    pub fn new_in(range_ppn: Range<PhysPageNum>, alloc: A) -> Self {
        Self{ range_ppn, alloc }
    }

    pub fn leak(mut self) -> Range<PhysPageNum> {
        let ret = self.range_ppn.clone();
        self.range_ppn.end = self.range_ppn.start;
        core::mem::forget(self);
        ret
    }
}

impl<A: FrameAllocatorHal> Drop for FrameTracker<A> {
    fn drop(&mut self) {
        self.alloc.dealloc(self.range_ppn.clone());
    }
}
