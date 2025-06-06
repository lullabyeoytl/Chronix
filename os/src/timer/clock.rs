//！ clocks
//! A clock may be system-wide and hence visible for all processes, 
//! or per-process if it measures time only within a single process.
//! more info refer to linux manual

use core::time::Duration;

/// the number of global clocks we need to record (no related to task)
pub const SUPPORT_CLOCK_NUM: usize = 2;

/// A settable system-wide clock that measures real (i.e., wall-clock) time.  
/// Setting this clock requires appropriate privileges.  
pub const CLOCK_REALTIME: usize = 0;

/// A nonsettable system-wide clock that represents monotonic time
/// since—as described by POSIX—"some unspecified point in the past".  
pub const CLOCK_MONOTONIC: usize = 1;

/// This is a clock that measures CPU time consumed by this
/// process (i.e., CPU time consumed by all threads in the process).  
/// On Linux, this clock is not settable.
pub const CLOCK_PROCESS_CPUTIME_ID: usize = 2;

/// This is a clock that measures CPU time consumed by this thread.  
/// On Linux, this clock is not settable.
pub const CLOCK_THREAD_CPUTIME_ID: usize = 3;

/// Similar to CLOCK_MONOTONIC, but provides access to a raw hardware-based time 
/// that is not subject to NTP adjustments 
/// or the incremental adjustments performed by adjtime(3).
pub const CLOCK_MONOTONIC_RAW: usize = 4;

/// A faster but less precise version of CLOCK_REALTIME. 
/// Use when you need very fast, but not fine-grained timestamps.
pub const CLOCK_REALTIME_COARSE: usize = 5;

/// A faster but less precise version of CLOCK_MONOTONIC. 
/// Use when you need very fast, but not fine-grained timestamps.
pub const CLOCK_MONOTONIC_COARSE: usize = 6;

/// global clocks
pub static mut CLOCK_DEVIATION: [Duration; SUPPORT_CLOCK_NUM] = [Duration::ZERO; SUPPORT_CLOCK_NUM];