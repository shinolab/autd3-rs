#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! A base library to drive AUTD3.

/// AUTD3 device.
pub mod autd3_device;
/// [`Datagram`] implementations.
///
/// [`Datagram`]: crate::datagram::Datagram
pub mod datagram;
/// Error module.
pub mod error;
/// Definitions for EtherCAT.
pub mod ethercat;
/// A module for working with firmware.
pub mod firmware;
#[doc(hidden)]
pub mod utils;

pub use autd3_core::{defined, geometry};
