use std::ffi::c_int;

use syscalls::{syscall, Errno, Sysno};

#[allow(non_camel_case_types)]
pub type pid_t = std::ffi::c_int;

pub const SCHED_NORMAL: u32 = 0;
pub const SCHED_FIFO: u32 = 1;
pub const SCHED_RR: u32 = 2;
pub const SCHED_BATCH: u32 = 3;
pub const SCHED_IDLE: u32 = 5;
pub const SCHED_DEADLINE: u32 = 6;
pub const SCHED_EXT: u32 = 7;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SchedAttr {
    /// Size of this structure
    pub size: u32, /* Size of this structure */
    /// Policy (SCHED_*)
    pub sched_policy: u32,
    /// Flags
    pub sched_flags: u64,
    /// Nice value (SCHED_OTHER, SCHED_BATCH)
    pub sched_nice: i32,

    /// Static priority (SCHED_FIFO, SCHED_RR)
    pub sched_priority: u32,

    /// For SCHED_DEADLINE
    pub sched_runtime: u64,
    /// For SCHED_DEADLINE
    pub sched_deadline: u64,
    /// For SCHED_DEADLINE
    pub sched_period: u64,

    /// Utilization hints
    pub sched_util_min: u32,
    /// Utilization hints
    pub sched_util_max: u32,
}
/// The sched_setattr() system call sets the scheduling policy and
/// associated attributes for the thread whose ID is specified in
/// `pid`. If `pid` equals zero, the scheduling policy and attributes of
/// the calling thread will be set.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_set_attr(pid: pid_t, attr: *mut SchedAttr, flags: u32) -> Result<usize, Errno> {
    syscall!(Sysno::sched_setattr, pid, attr, flags)
}
#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_get_attr(
    pid: pid_t,
    attr: *mut SchedAttr,
    size: u32,
    flags: u32,
) -> Result<usize, Errno> {
    syscall!(Sysno::sched_getattr, pid, attr, size, flags)
}

#[cfg(target_pointer_width = "32")]
const CPU_SET_SIZE: usize = 32;
#[cfg(target_pointer_width = "32")]
type Map = u32;
#[cfg(not(target_pointer_width = "32"))]
const CPU_SET_SIZE: usize = 16;
#[cfg(not(target_pointer_width = "32"))]
type Map = u64;

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct CpuSet {
    bits: [Map; CPU_SET_SIZE],
}
impl CpuSet {
    pub const fn empty() -> Self {
        Self {
            bits: [0; CPU_SET_SIZE],
        }
    }
    pub const fn full() -> Self {
        Self {
            bits: [Map::MAX; CPU_SET_SIZE],
        }
    }
    pub(crate) const fn as_raw(&self) -> *const CpuSet {
        self
    }

    pub(crate) const fn as_mut_raw(&mut self) -> *mut CpuSet {
        self
    }

    pub const fn set(self, core: usize) -> Self {
        let mut cs = self;
        let idx = core / size_of::<Map>();
        let bit = core % size_of::<Map>();
        cs.bits[idx] |= 1 << bit;
        cs
    }

    pub const fn clear(self, core: usize) -> Self {
        let mut cs = self;
        let idx = core / size_of::<Map>();
        let bit = core % size_of::<Map>();
        cs.bits[idx] &= 1 << bit;
        cs
    }

    pub const fn is_set(&mut self, core: usize) -> bool {
        let idx = core / size_of::<Map>();
        let bit = core % size_of::<Map>();
        self.bits[idx] & (1 << bit) > 0
    }

    pub const fn size_of() -> usize {
        size_of::<Self>()
    }
}

/// Sets the CPU affinity mask of the thread whose
/// ID is pid to the value specified by mask.  If pid is zero, then
/// the calling thread is used.  The argument cpusetsize is the length
/// (in bytes) of the data pointed to by mask.  Normally this argument
/// would be specified as sizeof(cpu_set_t).
///
/// If the thread specified by pid is not currently running on one of
/// the CPUs specified in mask, then that thread is migrated to one of
/// the CPUs specified in mask.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_set_affinity(
    pid: pid_t,
    cpusetsize: usize,
    mask: *const CpuSet,
) -> Result<usize, Errno> {
    syscall!(Sysno::sched_setaffinity, pid, cpusetsize, mask)
}

/// writes the affinity mask of the thread whose
/// ID is pid into the cpu_set_t structure pointed to by mask.  The
/// cpusetsize argument specifies the size (in bytes) of mask.  If pid
/// is zero, then the mask of the calling thread is returned.
///
/// Returns the number of bytes written to mask
#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_get_affinity(
    pid: pid_t,
    cpusetsize: usize,
    mask: *mut CpuSet,
) -> Result<usize, Errno> {
    syscall!(Sysno::sched_getaffinity, pid, cpusetsize, mask)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_yield() -> Result<usize, Errno> {
    syscall!(Sysno::sched_yield)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_get_priority_min(policy: c_int) -> Result<usize, Errno> {
    syscall!(Sysno::sched_get_priority_min, policy)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_get_priority_max(policy: c_int) -> Result<usize, Errno> {
    syscall!(Sysno::sched_get_priority_max, policy)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;

    #[test]
    fn set_attr() {
        let mut attr = SchedAttr {
            size: mem::size_of::<SchedAttr>() as u32,
            sched_policy: SCHED_IDLE,
            sched_flags: 0,
            sched_nice: 0,
            sched_priority: 0,
            sched_runtime: 0,
            sched_deadline: 0,
            sched_period: 0,
            sched_util_min: 0,
            sched_util_max: 0,
        };
        let ret = unsafe { sched_set_attr(0, &mut attr, 0) };
        assert_eq!(ret, Ok(0));

        let mut attr2 = unsafe { mem::zeroed::<SchedAttr>() };
        let ret = unsafe { sched_get_attr(0, &mut attr2, mem::size_of::<SchedAttr>() as u32, 0) };
        assert_eq!(ret, Ok(0));
        assert_eq!(attr2.sched_policy, { SCHED_IDLE });
    }

    #[test]
    fn test_cpuset() {
        let test = CpuSet::full();
        #[cfg(not(target_pointer_width = "32"))]
        assert_eq!(
            test,
            CpuSet {
                bits: [u64::MAX; 16]
            }
        );

        let test = CpuSet::empty();
        #[cfg(not(target_pointer_width = "32"))]
        assert_eq!(test, CpuSet { bits: [0; 16] });

        let test = CpuSet::empty().set(1);
        #[cfg(not(target_pointer_width = "32"))]
        assert_eq!(
            test,
            CpuSet {
                bits: [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            }
        );
    }

    #[test]
    fn test_affinity() {
        let mut cs_libc = unsafe { std::mem::zeroed() };
        unsafe { libc::CPU_ZERO(&mut cs_libc) };
        let x =
            unsafe { libc::sched_getaffinity(0, size_of_val(&cs_libc), &mut cs_libc as *mut _) };
        println!("{x}");
        // libc::cpu_set_t
        let mut cs = CpuSet::full();
        let ret = unsafe { sched_get_affinity(0, CpuSet::size_of(), cs.as_mut_raw()) };
        println!("{cs:?}");
        println!("{ret:?}");
        assert!(ret.is_ok());
        let ret = unsafe { sched_set_affinity(0, size_of::<CpuSet>(), CpuSet::full().as_raw()) };

        assert_eq!(ret, Ok(0))
    }
}
