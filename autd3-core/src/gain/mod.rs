mod drive;
mod emit_intensity;
mod error;
mod phase;

use std::collections::HashMap;

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
/// [`Gain`]: crate::datagram::Gain
pub trait GainContext: Send + Sync {
    /// Calculates the phase and intensity for the transducer.
    fn calc(&self, tr: &Transducer) -> Drive;
}

impl GainContext for Box<dyn GainContext> {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.as_ref().calc(tr)
    }
}

/// A trait for generating a context for the gain operation.
pub trait GainContextGenerator {
    /// The type of the context that actually performs the calculation.
    type Context: GainContext;

    /// Generate a context for the given device.
    fn generate(&mut self, device: &Device) -> Self::Context;
}

/// Trait for calculating the phase/amplitude of each transducer.
///
/// See also [`Gain`] derive macro.
///
/// [`Gain`]: autd3_derive::Gain
pub trait Gain: std::fmt::Debug {
    /// The type of the context generator.
    type G: GainContextGenerator;

    /// Initialize the gain and generate the context generator.
    ///
    /// `filter` is a hash map that holds a bit vector representing the indices of the enabled transducers for each device index.
    /// If `filter` is `None`, all transducers are enabled.
    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
    ) -> Result<Self::G, GainError>;
}

#[doc(hidden)]
pub struct GainOperationGenerator<G: GainContextGenerator> {
    pub generator: G,
    pub segment: Segment,
    pub transition: Option<TransitionMode>,
}

impl<G: GainContextGenerator> GainOperationGenerator<G> {
    pub fn new<T: Gain<G = G>>(
        gain: T,
        geometry: &Geometry,
        segment: Segment,
        transition: Option<TransitionMode>,
    ) -> Result<Self, GainError> {
        Ok(Self {
            generator: gain.init(geometry, None)?,
            segment,
            transition,
        })
    }
}

