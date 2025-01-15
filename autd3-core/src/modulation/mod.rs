mod error;
mod loop_behavior;
mod sampling_config;

use std::sync::Arc;

pub use error::{ModulationError, SamplingConfigError};
pub use loop_behavior::LoopBehavior;
pub use sampling_config::SamplingConfig;

use crate::datagram::{Segment, TransitionMode};

/// A trait to get the modulation property. (This trait is automatically implemented by the [`Modulation`] derive macro.)
///
/// [`Modulation`]: autd3_derive::Modulation
pub trait ModulationProperty {
    /// Get the sampling configuration.
    fn sampling_config(&self) -> SamplingConfig;
    /// Get the loop behavior.
    fn loop_behavior(&self) -> LoopBehavior;
}

/// Trait for applying amplitude modulation.
///
/// See also [`Modulation`] derive macro.
///
/// [`Modulation`]: autd3_derive::Modulation
pub trait Modulation: ModulationProperty + std::fmt::Debug {
    /// Calculate the modulation data.
    fn calc(self) -> Result<Vec<u8>, ModulationError>;
}

#[doc(hidden)]
pub struct ModulationOperationGenerator {
    pub g: Arc<Vec<u8>>,
    pub config: SamplingConfig,
    pub loop_behavior: LoopBehavior,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}
