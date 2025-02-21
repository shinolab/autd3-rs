mod drive;
mod emit_intensity;
mod error;
mod phase;

use std::collections::HashMap;

/// A bit vector type.
pub type BitVec = bit_vec::BitVec<u32>;

pub use drive::Drive;
pub use emit_intensity::EmitIntensity;
pub use error::GainError;
pub use phase::Phase;

use crate::{
    datagram::{Segment, TransitionMode},
    geometry::{Device, Geometry, Transducer},
};

/// A trait to calculate the phase and intensity for [`Gain`].
///
/// [`Gain`]: crate::gain::Gain
pub trait GainCalculator: Send + Sync {
    /// Calculates the phase and intensity for the transducer.
    #[must_use]
    fn calc(&self, tr: &Transducer) -> Drive;
}

impl GainCalculator for Box<dyn GainCalculator> {
    #[must_use]
    fn calc(&self, tr: &Transducer) -> Drive {
        self.as_ref().calc(tr)
    }
}

/// A trait for generating a calculator for the gain operation.
pub trait GainCalculatorGenerator {
    /// The type of the calculator that actually performs the calculation.
    type Calculator: GainCalculator;

    /// Generate a calculator for the given device.
    #[must_use]
    fn generate(&mut self, device: &Device) -> Self::Calculator;
}

/// Trait for calculating the phase/amplitude of each transducer.
///
/// See also [`Gain`] derive macro.
///
/// [`Gain`]: autd3_derive::Gain
pub trait Gain: std::fmt::Debug + Sized {
    /// The type of the calculator generator.
    type G: GainCalculatorGenerator;

    /// Initialize the gain and generate the calculator generator.
    fn init(self) -> Result<Self::G, GainError>;

    /// Initialize the gain and generate the calculator generator.
    ///
    /// `filter` is a hash map that holds a bit vector representing the indices of the enabled transducers for each device index.
    /// If `filter` is `None`, all transducers are enabled.
    fn init_full(
        self,
        _: &Geometry,
        _filter: Option<&HashMap<usize, BitVec>>,
        _parallel: bool,
    ) -> Result<Self::G, GainError> {
        self.init()
    }
}

#[doc(hidden)]
pub struct GainOperationGenerator<G: GainCalculatorGenerator> {
    pub generator: G,
    pub segment: Segment,
    pub transition: Option<TransitionMode>,
}

impl<G: GainCalculatorGenerator> GainOperationGenerator<G> {
    pub fn new<T: Gain<G = G>>(
        gain: T,
        geometry: &Geometry,
        segment: Segment,
        transition: Option<TransitionMode>,
        parallel: bool,
    ) -> Result<Self, GainError> {
        Ok(Self {
            generator: gain.init_full(geometry, None, parallel)?,
            segment,
            transition,
        })
    }
}
