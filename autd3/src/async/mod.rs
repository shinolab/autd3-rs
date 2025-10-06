/// Asynchronous [`Controller`] module.
pub mod controller;
mod sleeper;

pub use controller::Controller;
pub use sleeper::AsyncSleeper;
