// #[cfg(target_os = "linux")]
mod sched_linux;
// #[cfg(target_os = "linux")]
mod clock;
mod lowlevel;
pub use clock::*;
pub use lowlevel::clock::TimeSpec;
pub use lowlevel::sched::CpuSet;
pub use sched_linux::*;
