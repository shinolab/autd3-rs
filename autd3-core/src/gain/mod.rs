mod error;
mod filter;

use alloc::{boxed::Box, string::String, vec::Vec};
pub use error::GainError;
pub use filter::{DeviceTransducerMask, TransducerMask};

use crate::{
    datagram::DeviceMask,
    environment::Environment,
    firmware::Drive,
    firmware::{Segment, transition_mode::TransitionModeParams},
    geometry::{Device, Geometry, Transducer},
};

/// A trait to calculate the phase and intensity for [`Gain`].
///
/// [`Gain`]: crate::gain::Gain
pub trait GainCalculator<'a>: Send + Sync {
    /// Calculates the phase and intensity for the transducer.
    #[must_use]
    fn calc(&self, tr: &'a Transducer) -> Drive;
}

impl<'a> GainCalculator<'a> for Box<dyn GainCalculator<'a>> {
    fn calc(&self, tr: &'a Transducer) -> Drive {
        self.as_ref().calc(tr)
    }
}

/// A trait for generating a calculator for the gain operation.
pub trait GainCalculatorGenerator<'a> {
    /// The type of the calculator that actually performs the calculation.
    type Calculator: GainCalculator<'a>;

    /// Generate a calculator for the given device.
    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Self::Calculator;
}

/// Trait for calculating the phase/amplitude of each transducer.
///
/// See also [`Gain`] derive macro.
///
/// [`Gain`]: autd3_derive::Gain
pub trait Gain<'a>: core::fmt::Debug + Sized {
    /// The type of the calculator generator.
    type G: GainCalculatorGenerator<'a>;

    /// Initialize the gain and generate the calculator generator.
    ///
    /// `filter` is a hash map that holds a bit vector representing the indices of the enabled transducers for each device index.
    /// If `filter` is `None`, all transducers are enabled.
    fn init(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &TransducerMask,
    ) -> Result<Self::G, GainError>;
}

#[doc(hidden)]
pub struct GainOperationGenerator<'a, G> {
    pub generator: G,
    pub segment: Segment,
    pub transition_params: TransitionModeParams,
    pub __phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a, C: GainCalculatorGenerator<'a>> GainOperationGenerator<'a, C> {
    pub fn new<G: Gain<'a, G = C>>(
        gain: G,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Result<Self, GainError> {
        Ok(Self {
            generator: gain.init(geometry, env, &TransducerMask::from(filter))?,
            segment,
            transition_params,
            __phantom: core::marker::PhantomData,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
/// The result of the [`Gain`] inspection.
pub struct GainInspectionResult {
    /// The type name of the gain.
    pub name: String,
    /// The data of the gain.
    pub data: Vec<Drive>,
}
