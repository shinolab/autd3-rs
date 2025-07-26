mod error;

use std::sync::Arc;

use crate::firmware::{
    FirmwareLimits, SamplingConfig, Segment, transition_mode::TransitionModeParams,
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
    pub rep: u16,
    pub segment: Segment,
    pub transition_params: TransitionModeParams,
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
}
