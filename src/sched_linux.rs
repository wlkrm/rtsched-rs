use super::sched_attr::{sched_get_attr, sched_set_attr, SchedAttr};
use bitflags::bitflags;
pub use nix::sched::CpuSet;
pub use nix::unistd::Pid;
use std::{fmt::Error, mem};

/// Currently, Linux supports the scheduling policies defined in this enum.
#[derive(PartialEq, Debug)]
pub enum Policy {
    ///The standard round-robin time-sharing policy
    Other,
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
    ///  a round-robin policy.
    RoundRobin,
    ///a deadline scheduling policy;
    Deadline,
}

impl Policy {
    pub fn into_raw(self) -> u32 {
        match self {
            Policy::Batch => libc::SCHED_BATCH as u32,
            Policy::Deadline => libc::SCHED_DEADLINE as u32,
            Policy::Fifo => libc::SCHED_FIFO as u32,
            Policy::Idle => libc::SCHED_IDLE as u32,
            Policy::Other => libc::SCHED_OTHER as u32,
            Policy::RoundRobin => libc::SCHED_RR as u32,
        }
    }
    pub fn as_raw(&self) -> u32 {
        match self {
            Policy::Batch => libc::SCHED_BATCH as u32,
            Policy::Deadline => libc::SCHED_DEADLINE as u32,
            Policy::Fifo => libc::SCHED_FIFO as u32,
            Policy::Idle => libc::SCHED_IDLE as u32,
            Policy::Other => libc::SCHED_OTHER as u32,
            Policy::RoundRobin => libc::SCHED_RR as u32,
        }
    }
    pub fn from_raw(raw: u32) -> Result<Policy, Error> {
        match raw as i32 {
            libc::SCHED_OTHER => Ok(Policy::Other),
            libc::SCHED_FIFO => Ok(Policy::Fifo),
            libc::SCHED_RR => Ok(Policy::RoundRobin),
            libc::SCHED_BATCH => Ok(Policy::Batch),
            libc::SCHED_IDLE => Ok(Policy::Idle),
            libc::SCHED_DEADLINE => Ok(Policy::Deadline),
            _ => Err(Error),
        }
    }
}

bitflags! {
    /// These flags control the scheduling behavior:
    pub struct SchedFlags: libc::c_short {
        /// Children created by fork(2) do not inherit
        /// privileged scheduling policies.  See sched(7) for
        /// details.
        const SCHED_FLAG_RESET_ON_FORK = 0x01;
        ///This flag allows a SCHED_DEADLINE thread to reclaim bandwidth unused by other real-time threads.
        const  SCHED_FLAG_RECLAIM = 0x02;
        /// his flag allows an application to get informed
        /// about run-time overruns in SCHED_DEADLINE threads.
        /// Such overruns may be caused by (for example) coarse
        /// execution time accounting or incorrect parameter
        /// assignment.  Notification takes the form of a
        /// SIGXCPU signal which is generated on each overrun.
        /// This SIGXCPU signal is process-directed (see
        /// signal(7)) rather than thread-directed.  This is
        /// probably a bug.  On the one hand, sched_setattr()
        /// is being used to set a per-thread attribute.  On
        /// the other hand, if the process-directed signal is
        /// delivered to a thread inside the process other than
        /// the one that had a run-time overrun, the
        /// application has no way of knowing which thread
        /// overran.
        const SCHED_FLAG_DL_OVERRUN = 0x04;
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
    /// This  field specifies the static priority to be set when specifying `policy` as
    /// `Fifo` or `RoundRobin`.  The allowed range of priorities for these policies can  be
    /// determined  using  sched_get_priority_min(2)  and  sched_get_priority_max(2). For
    /// other `policy`, this field must be specified as 0.
    pub priority: u32,
    /// This field specifies the "Runtime" parameter for deadline scheduling. The value is
    /// expressed  in  nanoseconds.  This field is used only for the `Deadline` `policy`.
    pub runtime_ns: u64,
    /// This field specifies the "Deadline" parameter for deadline scheduling.   The  value
    /// is expressed in nanoseconds.  This field is used only for the `Deadline` `policy`.
    pub deadline_ns: u64,
    /// This  field specifies the "Period" parameter for deadline scheduling.  The value is
    /// expressed in nanoseconds. This field is used only for the `Deadline` `policy`.
    pub period_ns: u64,
}

/// The `get_attr()` function wraps the `sched_getattr()` system call and fetches the scheduling policy and
/// the associated attributes for the thread whose ID is specified in pid.
pub fn get_attr(pid: Pid) -> Result<Attributes, nix::Error> {
    let mut attr = SchedAttr {
        size: 0,
        sched_policy: 0,
        sched_flags: 0,
        sched_nice: 0,
        sched_priority: 0,
        sched_runtime: 0,
        sched_deadline: 0,
        sched_period: 0,
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
        0 => {
            let a = Attributes {
                policy: Policy::from_raw(attr.sched_policy).unwrap(),
                flags: SchedFlags::from_bits_truncate(attr.sched_flags as i16),
                nice: attr.sched_nice,
                priority: attr.sched_priority,
                deadline_ns: attr.sched_deadline,
                period_ns: attr.sched_period,
                runtime_ns: attr.sched_runtime,
            };
            Ok(a)
        }
        _ => Err(nix::Error::last()),
    }
}

/// The `set_attr()` function wraps the `sched_setattr()` system call and sets the scheduling policy and
/// associated attributes for the thread whose ID is specified in pid.
pub fn set_attr(pid: Pid, attr: Attributes) -> Result<(), nix::Error> {
    let mut attr = SchedAttr {
        size: mem::size_of::<SchedAttr>() as u32,
        sched_policy: attr.policy.into_raw(),
        sched_flags: attr.flags.bits() as u64,
        sched_nice: attr.nice,
        sched_priority: attr.priority,
        sched_runtime: attr.runtime_ns,
        sched_deadline: attr.deadline_ns,
        sched_period: attr.period_ns,
    };

    let ret = unsafe { sched_set_attr(pid.as_raw(), &mut attr, 0) };
    match ret {
        0 => Ok(()),
        _ => Err(nix::Error::last()),
    }
}
pub fn set_other(pid: Pid, nice: i32) -> Result<(), nix::Error> {
    let att_other = Attributes {
        policy: Policy::Other,
        nice,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority: 0,
        runtime_ns: 0,
    };
    set_attr(pid, att_other)
}
pub fn set_batch(pid: Pid, nice: i32) -> Result<(), nix::Error> {
    let att_batch = Attributes {
        policy: Policy::Batch,
        nice,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority: 0,
        runtime_ns: 0,
    };
    set_attr(pid, att_batch)
}
pub fn set_idle(pid: Pid) -> Result<(), nix::Error> {
    let att_batch = Attributes {
        policy: Policy::Idle,
        nice: 0,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority: 0,
        runtime_ns: 0,
    };
    set_attr(pid, att_batch)
}
pub fn set_fifo(pid: Pid, priority: u32) -> Result<(), nix::Error> {
    let att_batch = Attributes {
        policy: Policy::Fifo,
        nice: 0,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority,
        runtime_ns: 0,
    };
    set_attr(pid, att_batch)
}
pub fn set_rr(pid: Pid, priority: u32) -> Result<(), nix::Error> {
    let att_batch = Attributes {
        policy: Policy::RoundRobin,
        nice: 0,
        deadline_ns: 0,
        period_ns: 0,
        flags: SchedFlags::empty(),
        priority,
        runtime_ns: 0,
    };
    set_attr(pid, att_batch)
}
pub fn set_deadline(
    pid: Pid,
    deadline_ns: u64,
    period_ns: u64,
    runtime_ns: u64,
) -> Result<(), nix::Error> {
    if !((runtime_ns <= deadline_ns) && (deadline_ns <= period_ns)) {
        println!("Error: params are not sched_runtime <= sched_deadline <= sched_period!");
        return Err(nix::Error::EINVAL);
    };
    if runtime_ns < 1024 || deadline_ns < 1024 || period_ns < 1024 {
        println!("Error: params are y1024");
        return Err(nix::Error::EINVAL);
    }
    let att_batch = Attributes {
        policy: Policy::Deadline,
        nice: 0,
        deadline_ns,
        period_ns,
        flags: SchedFlags::empty(),
        priority: 0,
        runtime_ns,
    };
    set_attr(pid, att_batch)
}

pub fn get_priority_max(pol: Policy) -> Result<u32, nix::Error> {
    let ret = unsafe { libc::sched_get_priority_max(pol.into_raw() as i32) };
    match ret {
        -1 => Err(nix::Error::last()),
        n => Ok(n as u32),
    }
}

pub fn get_priority_min(pol: Policy) -> Result<u32, nix::Error> {
    let ret = unsafe { libc::sched_get_priority_min(pol.into_raw() as i32) };
    match ret {
        -1 => Err(nix::Error::last()),
        n => Ok(n as u32),
    }
}

pub fn sched_yield() -> Result<(), nix::Error> {
    let ret = unsafe { libc::sched_yield() };
    match ret {
        -1 => Err(nix::Error::last()),
        0 => Ok(()),
        _n => Err(nix::Error::last()),
    }
}

pub fn set_affinity(pida: Pid, set: &nix::sched::CpuSet) -> Result<(), nix::Error> {
    nix::sched::sched_setaffinity(pida, set)
}

pub fn get_affinity(pida: Pid) -> Result<CpuSet, nix::Error> {
    nix::sched::sched_getaffinity(pida)
}

#[cfg(test)]
mod tests {
    use crate::sched_linux::*;

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
        assert_eq!(a.policy, Policy::Other);
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

    #[test]
    fn test_affinity() {
        let mut set = get_affinity(Pid::this()).unwrap();
        set.unset(0).unwrap();
        set_affinity(Pid::this(), &set).unwrap();
        set.set(0).unwrap();
        set_affinity(Pid::this(), &set).unwrap();
        assert!(get_affinity(Pid::this()).unwrap().is_set(0).unwrap());
    }
}
