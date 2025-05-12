use syscalls::Errno;

use crate::lowlevel::clock::{
    clock_gettime, clock_nanosleep, clock_settime, clockid_t, TimeSpec, CLOCK_BOOTTIME,
    CLOCK_BOOTTIME_ALARM, CLOCK_MONOTONIC, CLOCK_MONOTONIC_COARSE, CLOCK_MONOTONIC_RAW,
    CLOCK_PROCESS_CPUTIME_ID, CLOCK_REALTIME, CLOCK_REALTIME_ALARM, CLOCK_REALTIME_COARSE,
    CLOCK_TAI, CLOCK_THREAD_CPUTIME_ID, TIMER_ABSTIME,
};

#[derive(Debug, Clone, Copy)]
pub enum ClockId {
    /// A settable system-wide clock that measures real (i.e., wall-
    /// clock) time.  Setting this clock requires appropriate privi‐
    /// leges.  This clock is affected by discontinuous jumps in the
    /// system time (e.g., if the system administrator manually
    /// changes the clock), and by the incremental adjustments per‐
    /// formed by adjtime(3) and NTP.
    ClockRealtime,

    /// (since Linux 3.0; Linux-specific)
    /// Like CLOCK_REALTIME, but not settable.  See timer_create(2)
    /// for further details.
    ClockRealtimeAlarm,

    /// (since Linux 2.6.32; Linux-specific)
    /// A faster but less precise version of CLOCK_REALTIME.  This
    /// clock is not settable.  Use when you need very fast, but not
    /// fine-grained timestamps.  Requires per-architecture support,
    /// and probably also architecture support for this flag in the
    /// vdso(7).
    ClockRealtimeCoarse,

    /// (since Linux 3.10; Linux-specific)
    /// A nonsettable system-wide clock derived from wall-clock time
    /// but ignoring leap seconds.  This clock does not experience
    /// discontinuities and backwards jumps caused by NTP inserting
    /// leap seconds as CLOCK_REALTIME does.
    ///
    /// The acronym TAI refers to International Atomic Time.
    ClockTai,
    /// A nonsettable system-wide clock that represents monotonic time
    /// since—as described by POSIX—"some unspecified point in the
    /// past".  On Linux, that point corresponds to the number of sec‐
    /// onds that the system has been running since it was booted.
    ///
    /// The CLOCK_MONOTONIC clock is not affected by discontinuous
    /// jumps in the system time (e.g., if the system administrator
    /// manually changes the clock), but is affected by the incremen‐
    /// tal adjustments performed by adjtime(3) and NTP.  This clock
    /// does not count time that the system is suspended.  All
    /// CLOCK_MONOTONIC variants guarantee that the time returned by
    /// consecutive calls will not go backwards, but successive calls
    /// may—depending on the architecture—return identical (not-
    /// increased) time values.
    ClockMonotonic,

    ///  (since Linux 2.6.32; Linux-specific)
    ///  A faster but less precise version of CLOCK_MONOTONIC.  Use
    /// when you need very fast, but not fine-grained timestamps.
    /// Requires per-architecture support, and probably also architec‐
    /// ture support for this flag in the vdso(7).
    ClockMonotonicCoarse,

    ///  (since Linux 2.6.28; Linux-specific)
    /// Similar to CLOCK_MONOTONIC, but provides access to a raw hard‐
    /// ware-based time that is not subject to NTP adjustments or the
    /// incremental adjustments performed by adjtime(3).  This clock
    /// does not count time that the system is suspended.
    ClockMonotonicRaw,

    /// (since Linux 2.6.39; Linux-specific)
    /// A nonsettable system-wide clock that is identical to
    /// CLOCK_MONOTONIC, except that it also includes any time that
    /// the system is suspended.  This allows applications to get a
    /// suspend-aware monotonic clock without having to deal with the
    /// complications of CLOCK_REALTIME, which may have discontinu‐
    /// ities if the time is changed using settimeofday(2) or similar.
    ClockBoottime,
    /// (since Linux 3.0; Linux-specific)
    /// Like CLOCK_BOOTTIME.  See timer_create(2) for further details.
    ClockBoottimeAlarm,
    /// (since Linux 2.6.12)
    /// This is a clock that measures CPU time consumed by this
    /// process (i.e., CPU time consumed by all threads in the
    /// process).  On Linux, this clock is not settable.
    ClockProcessCputimeId,

    ///  (since Linux 2.6.12)
    /// This is a clock that measures CPU time consumed by this
    /// thread.  On Linux, this clock is not settable.
    ClockThreadCputimeId,
}
impl ClockId {
    pub const fn as_raw(&self) -> clockid_t {
        match self {
            ClockId::ClockRealtime => CLOCK_REALTIME,
            ClockId::ClockRealtimeAlarm => CLOCK_REALTIME_ALARM,
            ClockId::ClockRealtimeCoarse => CLOCK_REALTIME_COARSE,
            ClockId::ClockTai => CLOCK_TAI,
            ClockId::ClockMonotonic => CLOCK_MONOTONIC,
            ClockId::ClockMonotonicCoarse => CLOCK_MONOTONIC_COARSE,
            ClockId::ClockMonotonicRaw => CLOCK_MONOTONIC_RAW,
            ClockId::ClockBoottime => CLOCK_BOOTTIME,
            ClockId::ClockBoottimeAlarm => CLOCK_BOOTTIME_ALARM,
            ClockId::ClockProcessCputimeId => CLOCK_PROCESS_CPUTIME_ID,
            ClockId::ClockThreadCputimeId => CLOCK_THREAD_CPUTIME_ID,
        }
    }
    pub const fn from_raw(clockid: clockid_t) -> Option<Self> {
        match clockid {
            CLOCK_REALTIME => Some(ClockId::ClockRealtime),
            CLOCK_REALTIME_ALARM => Some(ClockId::ClockRealtimeAlarm),
            CLOCK_REALTIME_COARSE => Some(ClockId::ClockRealtimeCoarse),
            CLOCK_TAI => Some(ClockId::ClockTai),
            CLOCK_MONOTONIC => Some(ClockId::ClockMonotonic),
            CLOCK_MONOTONIC_COARSE => Some(ClockId::ClockMonotonicCoarse),
            CLOCK_MONOTONIC_RAW => Some(ClockId::ClockMonotonicRaw),
            CLOCK_BOOTTIME => Some(ClockId::ClockBoottime),
            CLOCK_BOOTTIME_ALARM => Some(ClockId::ClockBoottimeAlarm),
            CLOCK_PROCESS_CPUTIME_ID => Some(ClockId::ClockProcessCputimeId),
            CLOCK_THREAD_CPUTIME_ID => Some(ClockId::ClockThreadCputimeId),
            _ => None,
        }
    }
}

pub fn get_time(clockid: ClockId) -> Result<TimeSpec, Errno> {
    let mut tp = TimeSpec::zeroed();
    unsafe { clock_gettime(clockid.as_raw(), &mut tp).and(Ok(tp)) }
}

pub fn set_time(clockid: ClockId, tp: TimeSpec) -> Result<(), Errno> {
    unsafe { clock_settime(clockid.as_raw(), &tp).and(Ok(())) }
}

pub fn nanosleep_relative(clockid: ClockId, tp: TimeSpec) -> Result<(), Errno> {
    unsafe { clock_nanosleep(clockid.as_raw(), 0, &tp, core::ptr::null_mut()).and(Ok(())) }
}
pub fn nanosleep_absolute(clockid: ClockId, tp: TimeSpec) -> Result<(), Errno> {
    unsafe {
        clock_nanosleep(clockid.as_raw(), TIMER_ABSTIME, &tp, core::ptr::null_mut()).and(Ok(()))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_time() {
        let time = get_time(ClockId::ClockBoottime).unwrap();
        assert!(time.tv_sec > 0);
    }

    #[test]
    fn test_sleep() {
        let time = nanosleep_relative(
            ClockId::ClockMonotonic,
            TimeSpec {
                tv_sec: 0,
                tv_nsec: 1_000_000,
            },
        )
        .unwrap();
        // assert!(time.tv_sec > 0);
    }
}
