mod error;
mod filter;

pub use error::GainError;
pub use filter::{DeviceTransducerMask, TransducerMask};

use crate::{
    datagram::DeviceMask,
    environment::Environment,
    firmware::Drive,
    firmware::{Segment, transition_mode::TransitionModeParams},
    geometry::{Device, Geometry, Transducer},
};

/// A trait to calculate the phase and intensity for each [`Transducer`].
///
/// [`Gain`]: crate::gain::Gain
pub trait GainCalculator<'a>: Send + Sync {
    /// Calculates the phase and intensity for the given [`Transducer`].
    #[must_use]
    fn calc(&self, tr: &'a Transducer) -> Drive;
}

impl<'a> GainCalculator<'a> for Box<dyn GainCalculator<'a>> {
    fn calc(&self, tr: &'a Transducer) -> Drive {
        self.as_ref().calc(tr)
    }
}

/// A trait for generating a [`GainCalculator`].
pub trait GainCalculatorGenerator<'a> {
    /// The type of [`GainCalculator`].
    type Calculator: GainCalculator<'a>;

    /// Generate a [`GainCalculator`] for the given [`Device`].
    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Self::Calculator;
}

/// A trait for intensity/phase calculation.
///
/// See also [`Gain`] derive macro.
///
/// [`Gain`]: autd3_derive::Gain
pub trait Gain<'a>: Sized {
    /// The type of [`GainCalculatorGenerator`].
    type G: GainCalculatorGenerator<'a>;

    /// Initialize the gain and generate [`GainCalculatorGenerator`].
    ///
    /// `filter` represents the enabled/disabled state of each transducer.
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
    /// The data of the gain.
    pub data: Vec<Drive>,
}
