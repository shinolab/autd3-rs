/// Asynchronous module.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub mod r#async;

/// A module for working with firmware operation.
pub mod operation;
/// A module for working with transmission control.
pub mod transmission;

mod driver;

pub use driver::Auto;

/// An estimated firmware version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    /// A driver for firmware version 10.
    V10,
    /// A driver for firmware version 11.
    V11,
    /// A driver for firmware version 12.
    V12,
}
