//! clock_gettime that never fails for a clock the host cannot answer.
//!
//! Rust's std unwraps clock_gettime, so one failing clock makes every engine
//! command panic. libriscv's macOS host compares the guest's Linux clock ids
//! against the host's own constants: CLOCK_MONOTONIC is 1 on Linux and 6 on
//! macOS, so it answers only CLOCK_REALTIME (0 on both). A guest that asks for
//! another clock falls back to the monotonic clock, then to the real-time one.

use core::ffi::c_void;

const CLOCK_REALTIME: i32 = 0;
const CLOCK_MONOTONIC: i32 = 1;

unsafe extern "C" {
	fn __real_clock_gettime(clock: i32, ts: *mut c_void) -> i32;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_clock_gettime(clock: i32, ts: *mut c_void) -> i32 {
	for c in [clock, CLOCK_MONOTONIC, CLOCK_REALTIME] {
		if unsafe { __real_clock_gettime(c, ts) } == 0 {
			return 0;
		}
	}
	-1
}
