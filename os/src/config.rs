//! Constants used in rCore
#[allow(unused)]

pub const USER_STACK_SIZE: usize = 1024 * 1024 * 8;
pub const KERNEL_STACK_SIZE: usize = 4096 * 16;
pub const KERNEL_HEAP_SIZE: usize = 0x30_00000;
pub const KERNEL_ADDR_OFFSET: usize = 0xffff_ffc0_0000_0000;

pub const KERNEL_STACK_TOP: usize = KERNEL_MEMORY_SPACE.1;
pub const KERNEL_STACK_BOTTOM: usize = KERNEL_STACK_TOP - KERNEL_STACK_SIZE;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAP_CONTEXT: usize = USER_MEMORY_SPACE.1 - PAGE_SIZE + 1;

#[allow(unused)]
pub const KERNEL_MEMORY_SPACE: (usize, usize) = (0xffff_ffc0_0000_0000, 0xffff_ffff_ffff_ffff);
pub const USER_MEMORY_SPACE: (usize, usize) = (0x0, 0x3f_ffff_ffff);

pub const USER_STACK_TOP: usize = TRAP_CONTEXT;

pub use crate::board::{CLOCK_FREQ, MEMORY_END, MMIO};

pub const BLOCK_SIZE: usize = 512;