#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! Core traits and types for AUTD3.

#[cfg_attr(docsrs, doc(cfg(feature = "acoustics")))]
#[cfg(feature = "acoustics")]
/// Utilities for acoustics.
pub mod acoustics;
#[cfg_attr(docsrs, doc(cfg(feature = "datagram")))]
#[cfg(feature = "datagram")]
/// Core traits for Datagram.
pub mod datagram;
#[cfg_attr(docsrs, doc(cfg(feature = "defined")))]
#[cfg(feature = "defined")]
/// Common constants and types.
pub mod defined;
#[cfg_attr(docsrs, doc(cfg(feature = "ethercat")))]
#[cfg(feature = "ethercat")]
/// Definitions for EtherCAT.
pub mod ethercat;
#[cfg_attr(docsrs, doc(cfg(feature = "gain")))]
#[cfg(feature = "gain")]
/// Core traits for Gain.
pub mod gain;
#[cfg_attr(docsrs, doc(cfg(feature = "geometry")))]
#[cfg(feature = "geometry")]
/// Geometry related modules.
pub mod geometry;
#[cfg_attr(docsrs, doc(cfg(feature = "link")))]
#[cfg(feature = "link")]
/// A interface to the device.
pub mod link;
#[cfg_attr(docsrs, doc(cfg(feature = "modulation")))]
#[cfg(feature = "modulation")]
/// Core traits for Modulation.
pub mod modulation;
#[cfg_attr(docsrs, doc(cfg(feature = "resampler")))]
#[cfg(feature = "resampler")]
/// Resampler module.
pub mod resampler;
#[cfg_attr(docsrs, doc(cfg(feature = "utils")))]
#[cfg(feature = "utils")]
#[doc(hidden)]
pub mod utils;

#[cfg_attr(docsrs, doc(cfg(feature = "async-trait")))]
#[cfg(feature = "async-trait")]
pub use async_trait::async_trait;

/// Utilities for user-defined [`Gain`] and [`Modulation`].
///
/// # Example
///
/// The following example shows how to define a custom [`Gain`] that generates a single focal point.
///
/// ```
/// use autd3_core::derive::*;
/// use autd3_core::geometry::Point3;
/// use autd3_core::defined::rad;
///
/// #[derive(Gain, Debug)]
/// pub struct FocalPoint {
///     pos: Point3,
/// }
///
/// pub struct Context {
///     pos: Point3,
///     wavenumber: f32,
/// }
///
/// impl GainContext for Context {
///     fn calc(&self, tr: &Transducer) -> Drive {
///         Drive {
///             phase: Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad),
///             intensity: EmitIntensity::MAX,
///         }
///     }
/// }
///
/// impl GainContextGenerator for FocalPoint {
///     type Context = Context;
///
///     fn generate(&mut self, device: &Device) -> Self::Context {
///         Context {
///             pos: self.pos,
///             wavenumber: device.wavenumber(),
///         }
///     }
/// }
///
/// impl Gain for FocalPoint {
///     type G = FocalPoint;
///
///     fn init(
///         self,
///         _geometry: &Geometry,
///         _filter: Option<&HashMap<usize, BitVec>>,
///     ) -> Result<Self::G, GainError> {
///         Ok(self)
///     }
/// }
/// ```
///
/// The following example shows how to define a modulation that outputs the maximum value only for a moment.
///
/// [`Modulation`] struct must have `config: SamplingConfig` and `loop_behavior: LoopBehavior`.
/// If you add `#[no_change]` attribute to `config`, you can't change the value of `config` except for the constructor.
///
/// ```
/// use autd3_core::defined::kHz;
/// use autd3_core::derive::*;
///
/// #[derive(Modulation, Debug)]
/// pub struct Burst {
///     config: SamplingConfig,
///     loop_behavior: LoopBehavior,
/// }
///
/// impl Burst {
///     pub fn new() -> Self {
///         Self {
///             config: SamplingConfig::new(4 * kHz).unwrap(),
///             loop_behavior: LoopBehavior::Infinite,
///         }
///     }
/// }
///
/// impl Modulation for Burst {
///     fn calc(self) -> Result<Vec<u8>, ModulationError>  {
///         Ok((0..4000)
///             .map(|i| if i == 3999 { u8::MAX } else { u8::MIN })
///             .collect())
///     }
/// }
/// ```
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
