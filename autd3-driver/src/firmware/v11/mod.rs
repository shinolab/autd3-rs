/// Asynchronous module.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub mod r#async;

/// A module for working with CPU firmware.
pub mod cpu;
/// A module for working with FPGA firmware.
pub mod fpga;
/// A module for working with firmware operation.
pub mod operation;
/// A module for working with transmission control.
pub mod transmission;

mod driver;

pub use driver::V11;
