#[cfg(not(feature = "tokio"))]
pub(crate) mod executor;
mod ext;
mod handler;
mod option;
mod smoothing;
mod status;
#[cfg(not(feature = "tokio"))]
mod timer;
mod utils;
#[cfg(any(target_os = "windows", not(feature = "tokio")))]
mod waker;
#[cfg(target_os = "windows")]
mod windows;

pub use handler::EtherCrabHandler;
pub use option::{EtherCrabOption, EtherCrabOptionFull};
pub use status::Status;
