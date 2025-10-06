mod error;

use crate::firmware::{SamplingConfig, Segment, transition_mode::TransitionModeParams};
use alloc::{sync::Arc, vec::Vec};
pub use error::ModulationError;

/// Trait for applying amplitude modulation.
///
/// See also [`Modulation`] derive macro.
///
/// [`Modulation`]: autd3_derive::Modulation
pub trait Modulation {
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
    pub rep: u16,
    pub segment: Segment,
    pub transition_params: TransitionModeParams,
}

#[derive(Debug, Clone, PartialEq)]
/// The result of the [`Modulation`] inspection.
pub struct ModulationInspectionResult {
    /// The data of the modulation.
    pub data: Vec<u8>,
    /// The sampling configuration.
    pub config: SamplingConfig,
}
