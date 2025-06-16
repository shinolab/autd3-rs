mod error;

use std::sync::Arc;

use crate::{
    datagram::{LoopBehavior, Segment, TransitionMode},
    derive::FirmwareLimits,
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
    fn calc(self, limits: &FirmwareLimits) -> Result<Vec<u8>, ModulationError>;

    /// The sampling configuration.
    #[must_use]
    fn sampling_config(&self) -> SamplingConfig;
}

#[doc(hidden)]
pub struct ModulationOperationGenerator {
    pub g: Arc<Vec<u8>>,
    pub config: SamplingConfig,
    pub limits: FirmwareLimits,
    pub loop_behavior: LoopBehavior,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}

#[derive(Debug, Clone, PartialEq)]
/// The result of the [`Modulation`] inspection.
pub struct ModulationInspectionResult {
    /// The type name of the modulation.
    pub name: String,
    /// The data of the modulation.
    pub data: Vec<u8>,
    /// The sampling configuration.
    pub config: SamplingConfig,
    /// The loop behavior.
    pub loop_behavior: LoopBehavior,
    /// The segment of the modulation.
    pub segment: Segment,
    /// The transition mode of the modulation.
    pub transition_mode: Option<TransitionMode>,
}
