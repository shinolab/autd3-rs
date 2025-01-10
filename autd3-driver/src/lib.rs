#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! A base library to drive AUTD3.

/// Utilities for acoustics.
pub mod acoustics;
/// AUTD3 device.
pub mod autd3_device;
/// [`Datagram`] implementations.
///
/// [`Datagram`]: crate::datagram::Datagram
pub mod datagram;
/// Common constants and types.
pub mod defined;
/// Error module.
pub mod error;
/// Definitions for EtherCAT.
pub mod ethercat;
/// A module for working with firmware.
pub mod firmware;
/// Geometry related modules.
pub mod geometry;
/// A interface to the device.
pub mod link;
#[doc(hidden)]
pub mod utils;

#[cfg(feature = "async-trait")]
pub use async_trait::async_trait;

/// Utilities for user-defined [`Gain`] and [`Modulation`].
///
/// [`Gain`]: crate::datagram::Gain
/// [`Modulation`]: crate::datagram::Modulation
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub mod derive {
    pub use crate::{
        datagram::{
            Datagram, DatagramS, Gain, GainContextGenerator, GainOperationGenerator, Modulation,
            ModulationOperationGenerator, ModulationProperty,
        },
        defined::DEFAULT_TIMEOUT,
        error::AUTDDriverError,
        firmware::{
            fpga::{
                Drive, EmitIntensity, LoopBehavior, Phase, SamplingConfig, Segment, TransitionMode,
            },
            operation::GainContext,
        },
        geometry::{Device, Geometry, Transducer},
    };
    pub use autd3_derive::{Builder, Gain, Modulation};
    pub use bit_vec::BitVec;
    pub use std::{collections::HashMap, sync::Arc};
    pub use tracing;
}
