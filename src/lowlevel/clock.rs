use std::ffi::c_int;

use syscalls::{syscall, Errno, Sysno};

#[allow(non_camel_case_types)]
pub type clockid_t = std::ffi::c_int;

pub const CLOCK_REALTIME: clockid_t = 0;
pub const CLOCK_MONOTONIC: clockid_t = 1;
pub const CLOCK_PROCESS_CPUTIME_ID: clockid_t = 2;
pub const CLOCK_THREAD_CPUTIME_ID: clockid_t = 3;
pub const CLOCK_MONOTONIC_RAW: clockid_t = 4;
pub const CLOCK_REALTIME_COARSE: clockid_t = 5;
pub const CLOCK_MONOTONIC_COARSE: clockid_t = 6;
pub const CLOCK_BOOTTIME: clockid_t = 7;
pub const CLOCK_REALTIME_ALARM: clockid_t = 8;
pub const CLOCK_BOOTTIME_ALARM: clockid_t = 9;
/// The driver implementing this got removed. The clock ID is kept as a
/// place holder. Do not reuse!
#[deprecated]
#[allow(dead_code)]
pub const CLOCK_SGI_CYCLE: clockid_t = 10;
pub const CLOCK_TAI: clockid_t = 11;

pub const TIMER_ABSTIME: c_int = 0x01;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TimeSpec {
    pub tv_sec: std::ffi::c_long,
    pub tv_nsec: std::ffi::c_long,
}
impl TimeSpec {
    pub const fn new() -> Self {
        Self::zeroed()
    }
    pub const fn zeroed() -> Self {
        Self {
            tv_sec: 0,
            tv_nsec: 0,
        }
    }

    pub const fn nanoseconds(nanoseconds: i64) -> Self {
        let (sec, nsec) = (nanoseconds / 1_000_000_000, nanoseconds % 1_000_000_000);
        Self {
            tv_sec: sec,
            tv_nsec: nsec,
        }
    }

    pub const fn as_nanoseconds(&self) -> i64 {
        self.tv_sec * 1_000_000_000 + self.tv_nsec
    }
    pub const fn as_microseconds(&self) -> i64 {
        self.tv_sec * 1_000_000 + self.tv_nsec / 1_000
    }
    pub const fn as_milliseconds(&self) -> i64 {
        self.tv_sec * 1_000 + self.tv_nsec / 1_000_000
    }
}

impl Default for TimeSpec {
    fn default() -> Self {
        Self::new()
    }
}

impl core::ops::Add for TimeSpec {
    type Output = TimeSpec;

    fn add(self, rhs: TimeSpec) -> TimeSpec {
        TimeSpec::nanoseconds(self.as_nanoseconds() + rhs.as_nanoseconds())
    }
}

impl core::ops::Sub for TimeSpec {
    type Output = TimeSpec;

    fn sub(self, rhs: TimeSpec) -> TimeSpec {
        TimeSpec::nanoseconds(self.as_nanoseconds() - rhs.as_nanoseconds())
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn clock_gettime(clockid: clockid_t, tp: *mut TimeSpec) -> Result<usize, Errno> {
    syscall!(Sysno::clock_gettime, clockid, tp)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn clock_settime(clockid: clockid_t, tp: *const TimeSpec) -> Result<usize, Errno> {
    syscall!(Sysno::clock_settime, clockid, tp)
}

#[allow(clippy::missing_safety_doc)]
/// # Parameter
///  * `remain` nullable
pub unsafe fn clock_nanosleep(
    clockid: clockid_t,
    flags: c_int,
    tp: *const TimeSpec,
    remain: *mut TimeSpec,
) -> Result<usize, Errno> {
    syscall!(Sysno::clock_nanosleep, clockid, flags, tp, remain)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nanos() {
        assert_eq!(
            TimeSpec::nanoseconds(0),
            TimeSpec {
                tv_sec: 0,
                tv_nsec: 0
            }
        );
        assert_eq!(
            TimeSpec::nanoseconds(-0),
            TimeSpec {
                tv_sec: 0,
                tv_nsec: 0
            }
        );

        assert_eq!(
            TimeSpec::nanoseconds(1_000_000_000),
            TimeSpec {
                tv_sec: 1,
                tv_nsec: 0
            }
        );
        assert_eq!(
            TimeSpec::nanoseconds(999_999_999),
            TimeSpec {
                tv_sec: 0,
                tv_nsec: 999_999_999
            }
        );
        assert_eq!(
            TimeSpec::nanoseconds(1_999_999_999),
            TimeSpec {
                tv_sec: 1,
                tv_nsec: 999_999_999
            }
        );

        assert_eq!(
            TimeSpec::nanoseconds(-1_999_999_999),
            TimeSpec {
                tv_sec: -1,
                tv_nsec: -999_999_999
            }
        );
    }
}
