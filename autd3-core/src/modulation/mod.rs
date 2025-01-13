mod error;
mod loop_behavior;
mod sampling_config;

pub use error::{ModulationError, SamplingConfigError};
pub use loop_behavior::LoopBehavior;
pub use sampling_config::SamplingConfig;

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
