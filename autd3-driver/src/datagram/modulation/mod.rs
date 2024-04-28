mod cache;
mod radiation_pressure;
mod segment;
mod transform;

use std::time::Duration;

pub use cache::Cache as ModulationCache;
pub use cache::IntoCache as IntoModulationCache;
pub use radiation_pressure::IntoRadiationPressure;
pub use radiation_pressure::RadiationPressure;
pub use segment::ChangeModulationSegment;
pub use transform::IntoTransform as IntoModulationTransform;
pub use transform::Transform as ModulationTransform;

use crate::defined::DEFAULT_TIMEOUT;
use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{EmitIntensity, LoopBehavior, SamplingConfiguration, Segment, TransitionMode},
        operation::{ModulationOp, NullOp},
    },
};

use super::{Datagram, DatagramS};

pub trait ModulationProperty {
    fn sampling_config(&self) -> SamplingConfiguration;
    fn loop_behavior(&self) -> LoopBehavior;
}

/// Modulation controls the amplitude modulation data.
///
/// Modulation has following restrictions:
/// * The buffer size is up to 65536.
/// * The sampling rate is [crate::firmware::fpga::fpga_clk_freq()]/N, where N is a 32-bit unsigned integer and must be at least [crate::fpga::SAMPLING_FREQ_DIV_MIN].
#[allow(clippy::len_without_is_empty)]
pub trait Modulation: ModulationProperty {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError>;
    fn len(&self) -> Result<usize, AUTDInternalError> {
        self.calc().map(|v| v.len())
    }
}

// GRCOV_EXCL_START
impl ModulationProperty for Box<dyn Modulation> {
    fn sampling_config(&self) -> SamplingConfiguration {
        self.as_ref().sampling_config()
    }

    fn loop_behavior(&self) -> LoopBehavior {
        self.as_ref().loop_behavior()
    }
}

impl Modulation for Box<dyn Modulation> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        self.as_ref().calc()
    }

    fn len(&self) -> Result<usize, AUTDInternalError> {
        self.as_ref().len()
    }
}

impl DatagramS for Box<dyn Modulation> {
    type O1 = ModulationOp;
    type O2 = NullOp;

    fn operation_with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((
            Self::O1::new(
                self.calc()?,
                self.sampling_config().division(),
                self.loop_behavior(),
                segment,
                transition_mode,
            ),
            Self::O2::default(),
        ))
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    use crate::derive::*;

    #[derive(Modulation, Clone, PartialEq, Debug)]
    pub struct TestModulation {
        pub buf: Vec<EmitIntensity>,
        pub config: SamplingConfiguration,
        pub loop_behavior: LoopBehavior,
    }

    impl Modulation for TestModulation {
        fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
            Ok(self.buf.clone())
        }
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfiguration::FREQ_4K_HZ)]
    fn test_sampling_config(#[case] config: SamplingConfiguration) {
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
                config: SamplingConfiguration::FREQ_4K_HZ,
                buf: vec![],
                loop_behavior,
            }
            .loop_behavior()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::n0(0)]
    #[case::n100(100)]
    fn test_len(#[case] len: usize) {
        assert_eq!(
            Ok(len),
            TestModulation {
                config: SamplingConfiguration::FREQ_4K_HZ,
                buf: vec![EmitIntensity::MIN; len],
                loop_behavior: LoopBehavior::infinite(),
            }
            .len()
        );
    }
}
