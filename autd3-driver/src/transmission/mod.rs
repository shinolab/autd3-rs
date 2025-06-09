/// Asynchronous module.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub mod r#async;

mod parallel_mode;
mod sender;
mod strategy;

pub use parallel_mode::ParallelMode;
pub use sender::{Sender, SenderOption};
pub use strategy::{FixedDelay, FixedSchedule, TimerStrategy};
