use crate::lowlevel::sched::{
    self, pid_t, sched_get_affinity, sched_get_attr, sched_set_affinity, sched_set_attr, CpuSet,
    SchedAttr, SCHED_BATCH, SCHED_DEADLINE, SCHED_EXT, SCHED_FIFO, SCHED_IDLE, SCHED_NORMAL,
    SCHED_RR,
};
use bitflags::bitflags;
use std::{ffi::c_int, fmt::Error, mem};
use syscalls::Errno;

/// Currently, Linux supports the scheduling policies defined in this enum.
#[derive(PartialEq, Debug)]
pub enum Policy {
    ///The standard round-robin time-sharing policy
    Normal,
    /// For "batch" style execution of processes
    Batch,
    /// For running very low priority background jobs
    Idle,
    /// Various "real-time" policies are also supported, for special
    /// time-critical applications that need precise control over the way
    /// in which runnable threads are selected for execution.
    /// The real-time policies that may be specified in policy
    /// are:a first-in, first-out policy; and
    Fifo,
    /// a round-robin policy.
    RoundRobin,
    /// a deadline scheduling policy;
    Deadline,
    Ext,
}

impl Policy {
    pub fn into_raw(self) -> u32 {
        self.as_raw()
    }
    pub fn as_raw(&self) -> u32 {
        match self {
            Policy::Batch => SCHED_BATCH,
            Policy::Deadline => SCHED_DEADLINE,
            Policy::Fifo => SCHED_FIFO,
            Policy::Idle => SCHED_IDLE,
            Policy::Normal => SCHED_NORMAL,
            Policy::RoundRobin => SCHED_RR,
            Policy::Ext => SCHED_EXT,
        }
    }
    pub fn from_raw(raw: u32) -> Result<Policy, Error> {
        match raw {
            SCHED_NORMAL => Ok(Policy::Normal),
            SCHED_FIFO => Ok(Policy::Fifo),
            SCHED_RR => Ok(Policy::RoundRobin),
            SCHED_BATCH => Ok(Policy::Batch),
            SCHED_IDLE => Ok(Policy::Idle),
            SCHED_DEADLINE => Ok(Policy::Deadline),
            SCHED_EXT => Ok(Policy::Ext),
            _ => Err(Error),
        }
    }
}

bitflags! {
    /// These flags control the scheduling behavior:
    pub struct SchedFlags: std::ffi::c_short {
        /// Children created by fork(2) do not inherit
        /// privileged scheduling policies. See sched(7) for
        /// details.
        const SCHED_FLAG_RESET_ON_FORK = 0x01;
        ///This flag allows a SCHED_DEADLINE thread to reclaim bandwidth unused by other real-time threads.
        const SCHED_FLAG_RECLAIM = 0x02;
        /// his flag allows an application to get informed
        /// about run-time overruns in SCHED_DEADLINE threads.
        /// Such overruns may be caused by (for example) coarse
        /// execution time accounting or incorrect parameter
        /// assignment. Notification takes the form of a
        /// SIGXCPU signal which is generated on each overrun.
        /// This SIGXCPU signal is process-directed (see
        /// signal(7)) rather than thread-directed. This is
        /// probably a bug. On the one hand, sched_setattr()
        /// is being used to set a per-thread attribute. On
        /// the other hand, if the process-directed signal is
        /// delivered to a thread inside the process other than
        /// the one that had a run-time overrun, the
        /// application has no way of knowing which thread
        /// overran.
        const SCHED_FLAG_DL_OVERRUN = 0x04;
        const SCHED_FLAG_KEEP_PARAMS = 0x10;
        /// These flags indicate that the sched_util_min or
        /// sched_util_max fields, respectively, are present,
        /// representing the expected minimum and maximum
        /// utilization of the thread.
        ///
        /// The utilization attributes provide the scheduler
        /// with boundaries within which it should schedule the
        /// thread, potentially informing its decisions
        /// regarding task placement and frequency selection.
        const SCHED_FLAG_UTIL_CLAMP_MIN = 0x20;
        /// See SCHED_FLAG_UTIL_CLAMP_MIN
        const SCHED_FLAG_UTIL_CLAMP_MAX	= 0x40;
    }
}

///Structure containing the scheduling policy and attributes for the specified thread.
pub struct Attributes {
    /// This field specifies the scheduling policy, as one of the values of the enum.
    pub policy: Policy,
    /// These flags control scheduling behavior:
    pub flags: SchedFlags,
    /// This field specifies the nice value to be set when
    /// specifying sched_policy as `SCHED_OTHER` or `SCHED_BATCH`.
    /// The nice value is a number in the range -20 (high
    /// priority) to +19 (low priority); see sched(7).
    pub nice: i32,
    /// This field specifies the static priority to be set when specifying `policy` as
    /// `Fifo` or `RoundRobin`. The allowed range of priorities for these policies can be
    /// determined using sched_get_priority_min(2) and sched_get_priority_max(2). For
    /// other `policy`, this field must be specified as 0.
    pub priority: u32,
    /// This field specifies the "Runtime" parameter for deadline scheduling. The value is
    /// expressed in nanoseconds. This field is used only for the `Deadline` `policy`.
    pub runtime_ns: u64,
    /// This field specifies the "Deadline" parameter for deadline scheduling. The value
    /// is expressed in nanoseconds. This field is used only for the `Deadline` `policy`.
    pub deadline_ns: u64,
    /// This field specifies the "Period" parameter for deadline scheduling. The value is
    /// expressed in nanoseconds. This field is used only for the `Deadline` `policy`.
    pub period_ns: u64,

    /// These fields specify the expected minimum and maximum utilization, respectively. They are ignored
    /// unless their corresponding SCHED_FLAG_UTIL_CLAMP_MIN or SCHED_FLAG_UTIL_CLAMP_MAX is set in sched_flags.
    ///
    /// Utilization is a value in the range [0, 1024], representing the percentage of CPU time used by a
    /// task when running at the maximum frequency on the highest capacity CPU of the system. This is a
    /// fixed point representation, where 1024 corresponds to 100%, and 0 corresponds to 0%. For example,
    /// a 20% utilization task is a task running for 2ms every 10ms at maximum frequency and is
    /// represented by a utilization value of 0.2 * 1024 = 205.
    ///
    /// A task with a minimum utilization value larger than 0 is more likely scheduled on a CPU with a
    /// capacity big enough to fit the specified value. A task with a maximum utilization value smaller
    /// than 1024 is more likely scheduled on a CPU with no more capacity than the specified value.
    ///
    /// A task utilization boundary can be reset by setting its field to UINT32_MAX (since Linux 5.11)
    pub sched_util_min: u32,
    /// See `sched_util_min`
    pub sched_util_max: u32,
}

pub struct Pid(pid_t);
impl Pid {
    pub fn as_raw(&self) -> pid_t {
        self.0
    }
    pub fn this() -> Self {
        Self(0)
    }
}

/// The `get_attr()` function wraps the `sched_getattr()` system call and fetches the scheduling policy and
/// the associated attributes for the thread whose ID is specified in pid.
pub fn get_attr(pid: Pid) -> Result<Attributes, Errno> {
    let mut attr = SchedAttr {
        size: 0,
        sched_policy: 0,
        sched_flags: 0,
        sched_nice: 0,
        sched_priority: 0,
        sched_runtime: 0,
        sched_deadline: 0,
        sched_period: 0,
        sched_util_min: 0,
        sched_util_max: 0,
    };

    let ret = unsafe {
        sched_get_attr(
            pid.as_raw(),
            &mut attr,
            mem::size_of::<SchedAttr>() as u32,
            0,
        )
    };
    match ret {
        Ok(_) => {
            let a = Attributes {
                policy: Policy::from_raw(attr.sched_policy).unwrap(),
                flags: SchedFlags::from_bits_truncate(attr.sched_flags as i16),
                nice: attr.sched_nice,
                priority: attr.sched_priority,
                deadline_ns: attr.sched_deadline,
                period_ns: attr.sched_period,
                runtime_ns: attr.sched_runtime,
                sched_util_min: attr.sched_util_min,
                sched_util_max: attr.sched_util_max,
            };
            Ok(a)
        }
        Err(err) => Err(err),
    }
}

/// The `set_attr()` function wraps the `sched_setattr()` system call and sets the scheduling policy and
/// associated attributes for the thread whose ID is specified in pid.
pub fn set_attr(pid: Pid, attr: Attributes) -> Result<(), Errno> {
    let mut attr = SchedAttr {
        size: mem::size_of::<SchedAttr>() as u32,
        sched_policy: attr.policy.into_raw(),
        sched_flags: attr.flags.bits() as u64,
        sched_nice: attr.nice,
        sched_priority: attr.priority,
        sched_runtime: attr.runtime_ns,
        sched_deadline: attr.deadline_ns,
        sched_period: attr.period_ns,
        sched_util_min: attr.sched_util_min,
        sched_util_max: attr.sched_util_max,
    };

    unsafe { sched_set_attr(pid.as_raw(), &mut attr, 0) }.and(Ok(()))
}
pub fn set_other(pid: Pid, nice: i32) -> Result<(), Errno> {
    let att_other = Attributes {
        policy: Policy::Normal,
        nice,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority: 0,
        runtime_ns: 0,
        sched_util_min: 0,
        sched_util_max: 0,
    };
    set_attr(pid, att_other)
}
pub fn set_batch(pid: Pid, nice: i32) -> Result<(), Errno> {
    let att_batch = Attributes {
        policy: Policy::Batch,
        nice,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority: 0,
        runtime_ns: 0,
        sched_util_min: 0,
        sched_util_max: 0,
    };
    set_attr(pid, att_batch)
}
pub fn set_idle(pid: Pid) -> Result<(), Errno> {
    let att_batch = Attributes {
        policy: Policy::Idle,
        nice: 0,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority: 0,
        runtime_ns: 0,
        sched_util_min: 0,
        sched_util_max: 0,
    };
    set_attr(pid, att_batch)
}
pub fn set_fifo(pid: Pid, priority: u32) -> Result<(), Errno> {
    let att_batch = Attributes {
        policy: Policy::Fifo,
        nice: 0,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority,
        runtime_ns: 0,
        sched_util_min: 0,
        sched_util_max: 0,
    };
    set_attr(pid, att_batch)
}
pub fn set_rr(pid: Pid, priority: u32) -> Result<(), Errno> {
    let att_batch = Attributes {
        policy: Policy::RoundRobin,
        nice: 0,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority,
        runtime_ns: 0,
        sched_util_min: 0,
        sched_util_max: 0,
    };
    set_attr(pid, att_batch)
}
pub fn set_deadline(
    pid: Pid,
    deadline_ns: u64,
    period_ns: u64,
    runtime_ns: u64,
) -> Result<(), Errno> {
    if !((runtime_ns <= deadline_ns) && (deadline_ns <= period_ns)) {
        println!("Error: params are not sched_runtime <= sched_deadline <= sched_period!");
        return Err(Errno::EINVAL);
    };
    if runtime_ns < 1024 || deadline_ns < 1024 || period_ns < 1024 {
        println!("Error: params are y1024");
        return Err(Errno::EINVAL);
    }
    let att_batch = Attributes {
        policy: Policy::Deadline,
        nice: 0,
        deadline_ns,
        period_ns,
        flags: SchedFlags::empty(),
        priority: 0,
        runtime_ns,
        sched_util_min: 0,
        sched_util_max: 0,
    };
    set_attr(pid, att_batch)
}

pub fn get_priority_max(pol: Policy) -> Result<usize, Errno> {
    unsafe { sched::sched_get_priority_max(pol.into_raw() as c_int) }
}

pub fn get_priority_min(pol: Policy) -> Result<usize, Errno> {
    unsafe { sched::sched_get_priority_min(pol.into_raw() as c_int) }
}

pub fn sched_yield() -> Result<(), Errno> {
    unsafe { sched::sched_yield() }.and(Ok(()))
}

pub fn set_affinity(pid: Pid, set: CpuSet) -> Result<(), Errno> {
    unsafe { sched_set_affinity(pid.as_raw(), CpuSet::size_of(), set.as_raw()).and(Ok(())) }
}

pub fn get_affinity(pid: Pid) -> Result<CpuSet, Errno> {
    let mut cpuset = CpuSet::empty();
    unsafe { sched_get_affinity(pid.as_raw(), CpuSet::size_of(), cpuset.as_mut_raw()) }
        .and(Ok(cpuset))
}

#[cfg(test)]
mod tests {
    use crate::sched::*;

    #[test]
    fn test_setattr() {
        let att = Attributes {
            policy: Policy::Batch,
            nice: 4,
            deadline_ns: 0,
            period_ns: 0,
            flags: SchedFlags::empty(),
            priority: 0,
            runtime_ns: 0,
            sched_util_min: 0,
            sched_util_max: 0,
        };
        set_attr(Pid::this(), att).unwrap();
        let a = get_attr(Pid::this()).unwrap();

        assert_eq!(a.policy, Policy::Batch);
        assert_eq!(a.nice, 4);
    }
    #[test]
    fn test_setter() {
        set_other(Pid::this(), -20).unwrap();
        let a = get_attr(Pid::this()).unwrap();
        assert_eq!(a.policy, Policy::Normal);
        assert_eq!(a.nice, -20);

        set_batch(Pid::this(), 19).unwrap();
        let a = get_attr(Pid::this()).unwrap();
        assert_eq!(a.policy, Policy::Batch);
        assert_eq!(a.nice, 19);

        set_idle(Pid::this()).unwrap();
        let a = get_attr(Pid::this()).unwrap();
        assert_eq!(a.policy, Policy::Idle);

        set_fifo(Pid::this(), 60).unwrap();
        let a = get_attr(Pid::this()).unwrap();
        assert_eq!(a.policy, Policy::Fifo);
        assert_eq!(a.nice, 0);
        assert_eq!(a.priority, 60);

        set_rr(Pid::this(), 98).unwrap();
        let a = get_attr(Pid::this()).unwrap();
        assert_eq!(a.policy, Policy::RoundRobin);
        assert_eq!(a.nice, 0);
        assert_eq!(a.priority, 98);

        set_deadline(Pid::this(), 1_000_000, 1_000_000, 50_000).unwrap();
        let a = get_attr(Pid::this()).unwrap();
        assert_eq!(a.policy, Policy::Deadline);
        assert_eq!(a.nice, 0);
        assert_eq!(a.priority, 0);
        assert_eq!(a.deadline_ns, 1_000_000);
        assert_eq!(a.period_ns, 1_000_000);
        assert_eq!(a.runtime_ns, 50_000);
    }

    #[test]
    fn test_deadline() {
        set_deadline(Pid::this(), 1_000_000, 1_000_000, 50_000).unwrap();
        let a = get_attr(Pid::this()).unwrap();
        assert_eq!(a.policy, Policy::Deadline);
        assert_eq!(a.nice, 0);
        assert_eq!(a.priority, 0);
        assert_eq!(a.deadline_ns, 1_000_000);
        assert_eq!(a.period_ns, 1_000_000);
        assert_eq!(a.runtime_ns, 50_000);
        sched_yield().unwrap();
    }

    #[test]
    fn test_prio() {
        get_priority_max(Policy::Fifo).unwrap();
        get_priority_min(Policy::Fifo).unwrap();
    }

    // #[test]
    // fn test_affinity() {
    //     let mut set = get_affinity(Pid::this()).unwrap();
    //     set.unset(0).unwrap();
    //     set_affinity(Pid::this(), &set).unwrap();
    //     set.set(0).unwrap();
    //     set_affinity(Pid::this(), &set).unwrap();
    //     assert!(get_affinity(Pid::this()).unwrap().is_set(0).unwrap());
    // }
}
