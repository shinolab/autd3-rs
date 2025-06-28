pub(crate) mod tag;

/// A driver module with firmware version auto-detection
pub mod auto;
/// A driver module for firmware version 10
pub mod v10;
/// A driver module for firmware version 11
pub mod v11;
/// A driver module for firmware version 12
pub mod v12;
/// A driver module for firmware version 12.1
pub mod v12_1;

/// A alias module for the latest firmware
pub mod latest {
    pub use super::v12_1::*;
    pub use V12_1 as Latest;
}

/// Firmware version
pub mod version;

/// A module for firmware driver
pub mod driver;
