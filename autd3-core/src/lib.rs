#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! Core traits and types for AUTD3.

#[cfg(feature = "acoustics")]
/// Utilities for acoustics.
pub mod acoustics;
#[cfg(feature = "datagram")]
/// Core traits for Datagram.
pub mod datagram;
#[cfg(feature = "defined")]
/// Common constants and types.
pub mod defined;
#[cfg(feature = "ethercat")]
/// Definitions for EtherCAT.
pub mod ethercat;
#[cfg(feature = "gain")]
/// Core traits for Gain.
pub mod gain;
#[cfg(feature = "geometry")]
/// Geometry related modules.
pub mod geometry;
#[cfg(feature = "link")]
/// A interface to the device.
pub mod link;
#[cfg(feature = "modulation")]
/// Core traits for Modulation.
pub mod modulation;
#[cfg(feature = "resampler")]
/// Resampler module.
pub mod resampler;
#[cfg(feature = "utils")]
#[doc(hidden)]
pub mod utils;

#[cfg(feature = "async-trait")]
pub use async_trait::async_trait;

/// Utilities for user-defined [`Gain`] and [`Modulation`].
///
/// [`Gain`]: crate::gain::Gain
/// [`Modulation`]: crate::modulation::Modulation
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub mod derive {
    #[cfg(any(feature = "gain", feature = "modulation"))]
    mod common {
        pub use crate::datagram::{DatagramS, Segment, TransitionMode};
        pub use crate::{defined::DEFAULT_TIMEOUT, geometry::Geometry};
        pub use tracing;
    }
    #[cfg(any(feature = "gain", feature = "modulation"))]
    pub use common::*;

    #[cfg(feature = "gain")]
    mod gain {
        pub use crate::gain::{
            BitVec, Drive, EmitIntensity, Gain, GainContext, GainContextGenerator, GainError,
            GainOperationGenerator, Phase,
        };
        pub use crate::geometry::{Device, Transducer};
        pub use autd3_derive::Gain;
    }
    #[cfg(feature = "gain")]
    pub use gain::*;

    #[cfg(feature = "modulation")]
    mod modulation {
        pub use crate::modulation::{
            LoopBehavior, Modulation, ModulationError, ModulationOperationGenerator,
            ModulationProperty, SamplingConfig,
        };
        pub use autd3_derive::Modulation;
        pub use std::{collections::HashMap, sync::Arc};
    }
    #[cfg(feature = "modulation")]
    pub use modulation::*;
}
