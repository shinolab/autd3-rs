#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! Core traits and types for AUTD3.

#[cfg_attr(docsrs, doc(cfg(feature = "acoustics")))]
#[cfg(feature = "acoustics")]
/// Utilities for acoustics.
pub mod acoustics;
#[cfg_attr(docsrs, doc(cfg(feature = "common")))]
#[cfg(feature = "common")]
/// Common constants and types.
pub mod common;
#[cfg_attr(docsrs, doc(cfg(feature = "datagram")))]
#[cfg(feature = "datagram")]
/// Core traits for Datagram.
pub mod datagram;
#[cfg_attr(docsrs, doc(cfg(feature = "environment")))]
#[cfg(feature = "environment")]
#[doc(hidden)]
pub mod environment;
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
#[cfg_attr(docsrs, doc(cfg(feature = "sampling_config")))]
#[cfg(feature = "sampling_config")]
#[doc(hidden)]
pub mod sampling_config;
#[cfg_attr(docsrs, doc(cfg(feature = "sleep")))]
#[cfg(feature = "sleep")]
#[doc(hidden)]
pub mod sleep;
#[cfg_attr(docsrs, doc(cfg(feature = "utils")))]
#[cfg(feature = "utils")]
#[doc(hidden)]
pub mod utils;

/// Utilities for user-common [`Gain`] and [`Modulation`].
///
/// # Example
///
/// The following example shows how to define a custom [`Gain`] that generates a single focal point.
///
/// ```
/// use autd3_core::derive::*;
/// use autd3_core::geometry::Point3;
/// use autd3_core::common::rad;
///
/// #[derive(Gain, Debug)]
/// pub struct FocalPoint {
///     pos: Point3,
/// }
///
/// #[derive(Clone, Copy)]
/// pub struct Impl {
///     pos: Point3,
///     wavenumber: f32,
/// }
///
/// impl<'a> GainCalculator<'a> for Impl {
///     fn calc(&self, tr: &'a Transducer) -> Drive {
///         Drive {
///             phase: Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad),
///             intensity: Intensity::MAX,
///         }
///     }
/// }
///
/// impl<'a> GainCalculatorGenerator<'a> for Impl {
///     type Calculator = Self;
///
///     fn generate(&mut self, _: &'a Device) -> Self::Calculator {
///        *self
///     }
/// }
///
/// impl<'a> Gain<'a> for FocalPoint {
///     type G = Impl;
///
///     fn init(
///         self,
///         _geometry: &'a Geometry,
///         env: &Environment,
///         _filter: &TransducerFilter,
///     ) -> Result<Self::G, GainError> {
///         Ok(Impl {
///             pos: self.pos,
///             wavenumber: env.wavenumber(),
///         })
///     }
/// }
/// ```
///
/// The following example shows how to define a modulation that outputs the maximum value only for a moment.
///
/// ```
/// use autd3_core::common::kHz;
/// use autd3_core::derive::*;
///
/// #[derive(Modulation, Debug)]
/// pub struct Burst {
/// }
///
/// impl Burst {
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
///
/// impl Modulation for Burst {
///     fn calc(self, _limits: &FirmwareLimits) -> Result<Vec<u8>, ModulationError>  {
///         Ok((0..4000)
///             .map(|i| if i == 3999 { u8::MAX } else { u8::MIN })
///             .collect())
///     }
///
///     fn sampling_config(&self) -> SamplingConfig {
///         SamplingConfig::new(4. * kHz)
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
        pub use crate::{
            datagram::{
                DatagramOption, DeviceFilter, FirmwareLimits, Inspectable, InspectionResult,
                Segment, internal, transition_mode,
            },
            environment::Environment,
            geometry::Geometry,
        };
        pub use tynm;
    }
    #[cfg(any(feature = "gain", feature = "modulation"))]
    pub use common::*;

    #[cfg(feature = "gain")]
    mod gain {
        pub use crate::{
            datagram::DatagramS,
            gain::{
                Drive, Gain, GainCalculator, GainCalculatorGenerator, GainError,
                GainInspectionResult, GainOperationGenerator, Intensity, Phase, TransducerFilter,
            },
            geometry::{Device, Transducer},
        };
        pub use autd3_derive::Gain;
        pub use num_cpus;
        pub use std::collections::HashMap;
    }
    #[cfg(feature = "gain")]
    pub use gain::*;

    #[cfg(feature = "modulation")]
    mod modulation {
        pub use crate::datagram::{Datagram, DatagramL};
        pub use crate::modulation::{
            Modulation, ModulationError, ModulationInspectionResult, ModulationOperationGenerator,
        };
        pub use crate::sampling_config::{SamplingConfig, SamplingConfigError};
        pub use autd3_derive::Modulation;
    }
    #[cfg(feature = "modulation")]
    pub use modulation::*;
}
