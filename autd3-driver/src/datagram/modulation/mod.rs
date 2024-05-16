mod cache;
mod radiation_pressure;
mod transform;

use std::time::Duration;

pub use cache::Cache as ModulationCache;
pub use cache::IntoCache as IntoModulationCache;
pub use radiation_pressure::IntoRadiationPressure;
pub use radiation_pressure::RadiationPressure;
pub use transform::IntoTransform as IntoModulationTransform;
pub use transform::Transform as ModulationTransform;

use crate::defined::DEFAULT_TIMEOUT;
use crate::derive::EmitIntensity;
use crate::derive::Geometry;
use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::{ModulationOp, NullOp},
    },
};

use super::DatagramST;

pub trait ModulationProperty {
    fn sampling_config(&self) -> SamplingConfig;
    fn loop_behavior(&self) -> LoopBehavior;
}

/// Modulation controls the amplitude modulation data.
///
/// Modulation has following restrictions:
/// * The buffer size is up to 65536.
/// * The sampling rate is [crate::firmware::fpga::fpga_clk_freq()]/N, where N is a 32-bit unsigned integer and must be at least [crate::fpga::SAMPLING_FREQ_DIV_MIN].
#[allow(clippy::len_without_is_empty)]
pub trait Modulation: ModulationProperty {
    fn calc(&self, geometry: &Geometry) -> Result<Vec<EmitIntensity>, AUTDInternalError>;
}

// GRCOV_EXCL_START
impl ModulationProperty for Box<dyn Modulation> {
    fn sampling_config(&self) -> SamplingConfig {
        self.as_ref().sampling_config()
    }

    fn loop_behavior(&self) -> LoopBehavior {
        self.as_ref().loop_behavior()
    }
}

impl Modulation for Box<dyn Modulation> {
    fn calc(&self, geometry: &Geometry) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        self.as_ref().calc(geometry)
    }
}

impl DatagramST for Box<dyn Modulation> {
    type O1 = ModulationOp<Self>;
    type O2 = NullOp;

    fn operation_with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(self, segment, transition_mode),
            Self::O2::default(),
        )
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{defined::kHz, derive::*};

    #[derive(Modulation, Clone, PartialEq, Debug)]
    pub struct TestModulation {
        pub buf: Vec<EmitIntensity>,
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl Modulation for TestModulation {
        fn calc(&self, _: &Geometry) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
            Ok(self.buf.clone())
        }
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::Freq(4 * kHz))]
    fn test_sampling_config(#[case] config: SamplingConfig) {
        assert_eq!(
            config,
            TestModulation {
                config,
                buf: vec![],
                loop_behavior: LoopBehavior::infinite(),
            }
            .sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::once(LoopBehavior::once())]
    fn test_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            TestModulation {
                config: SamplingConfig::Freq(4 * kHz),
                buf: vec![],
                loop_behavior,
            }
            .loop_behavior()
        );
    }
}
