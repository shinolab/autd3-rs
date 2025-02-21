mod error;

use std::sync::Arc;

use crate::{
    datagram::{LoopBehavior, Segment, TransitionMode},
    sampling_config::SamplingConfig,
};
pub use error::ModulationError;

/// Trait for applying amplitude modulation.
///
/// See also [`Modulation`] derive macro.
///
/// [`Modulation`]: autd3_derive::Modulation
pub trait Modulation: std::fmt::Debug {
    /// Calculate the modulation data.
    fn calc(self) -> Result<Vec<u8>, ModulationError>;

    /// The sampling configuration.
    #[must_use]
    fn sampling_config(&self) -> SamplingConfig;
}

#[doc(hidden)]
pub struct ModulationOperationGenerator {
    pub g: Arc<Vec<u8>>,
    pub config: SamplingConfig,
    pub loop_behavior: LoopBehavior,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}
