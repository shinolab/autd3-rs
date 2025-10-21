#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! A base library to drive AUTD3.

/// [`Datagram`] implementations.
///
/// [`Datagram`]: autd3_core::datagram::Datagram
pub mod datagram;
/// Error module.
pub mod error;
/// A module for working with firmware.
pub mod firmware;

pub use autd3_core::{common, devices, ethercat, geometry, link};

#[cfg(test)]
pub(crate) mod tests {
    use autd3_core::{
        derive::{Device, Geometry},
        devices::AUTD3,
    };

    pub fn create_device() -> Device {
        AUTD3::default().into()
    }

    pub fn create_geometry(n: usize) -> crate::geometry::Geometry {
        Geometry::new((0..n).map(|_| create_device()).collect())
    }
}
