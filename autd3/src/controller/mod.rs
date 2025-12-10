#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
mod r#async;
mod blocking;

#[cfg(feature = "async")]
pub use r#async::{AsyncController, AsyncSender};
#[cfg(feature = "async")]
pub use autd3_core::sleep::AsyncSleeper;
pub use autd3_driver::firmware::transmission::{ParallelMode, SenderOption};
pub use blocking::{Controller, Sender};
