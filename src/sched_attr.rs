use libc::{self, c_long, pid_t};

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
    pub sched_priority: u32, /*  */
    /// For SCHED_DEADLINE
    pub sched_runtime: u64,
    /// For SCHED_DEADLINE
    pub sched_deadline: u64,
    /// For SCHED_DEADLINE
    pub sched_period: u64,
}
/// The sched_setattr() system call sets the scheduling policy and
/// associated attributes for the thread whose ID is specified in
/// `pid`. If `pid` equals zero, the scheduling policy and attributes of
/// the calling thread will be set.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_set_attr(pid: pid_t, attr: *mut SchedAttr, flags: u32) -> c_long {
    libc::syscall(libc::SYS_sched_setattr, pid, attr, flags)
}
#[allow(clippy::missing_safety_doc)]
pub unsafe fn sched_get_attr(pid: pid_t, attr: *mut SchedAttr, size: u32, flags: u32) -> c_long {
    libc::syscall(libc::SYS_sched_getattr, pid, attr, size, flags)
}
#[cfg(test)]
mod test {
    use std::mem;

    use libc::SCHED_IDLE;

    use crate::sched_attr::sched_get_attr;

    use super::{sched_set_attr, SchedAttr};

    #[test]
    fn set_attr() {
        let mut attr = SchedAttr {
            size: mem::size_of::<SchedAttr>() as u32,
            sched_policy: SCHED_IDLE as u32,
            sched_flags: 0,
            sched_nice: 0,
            sched_priority: 0,
            sched_runtime: 0,
            sched_deadline: 0,
            sched_period: 0,
        };
        let ret = unsafe { sched_set_attr(0, &mut attr, 0) };
        assert_eq!(ret, 0);

        let mut attr2 = unsafe { mem::zeroed::<SchedAttr>() };
        let ret = unsafe { sched_get_attr(0, &mut attr2, mem::size_of::<SchedAttr>() as u32, 0) };
        assert_eq!(ret, 0);
        assert_eq!(attr2.sched_policy, SCHED_IDLE as u32);
    }
}
