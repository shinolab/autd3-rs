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

use crate::operation::ModulationOp;
use crate::operation::NullOp;
use crate::{
    common::{EmitIntensity, LoopBehavior, SamplingConfiguration, Segment},
    error::AUTDInternalError,
};

use super::Datagram;
use super::DatagramS;

pub trait ModulationProperty {
    fn sampling_config(&self) -> SamplingConfiguration;
    fn loop_behavior(&self) -> LoopBehavior;
}

/// Modulation controls the amplitude modulation data.
///
/// Modulation has following restrictions:
/// * The buffer size is up to 65536.
/// * The sampling rate is [crate::fpga::FPGA_CLK_FREQ]/N, where N is a 32-bit unsigned integer and must be at least [crate::fpga::SAMPLING_FREQ_DIV_MIN].
#[allow(clippy::len_without_is_empty)]
pub trait Modulation: ModulationProperty {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError>;
    fn len(&self) -> Result<usize, AUTDInternalError> {
        self.calc().map(|v| v.len())
    }
}

impl ModulationProperty for Box<dyn Modulation> {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn sampling_config(&self) -> SamplingConfiguration {
        self.as_ref().sampling_config()
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn loop_behavior(&self) -> LoopBehavior {
        self.as_ref().loop_behavior()
    }
}

impl Modulation for Box<dyn Modulation> {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        self.as_ref().calc()
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn len(&self) -> Result<usize, AUTDInternalError> {
        self.as_ref().len()
    }
}

impl DatagramS for Box<dyn Modulation> {
    type O1 = ModulationOp;
    type O2 = NullOp;

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn operation_with_segment(
        self,
        segment: Segment,
        update_segment: bool,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        let freq_div = self.sampling_config().frequency_division();
        Ok((
            Self::O1::new(
                self.calc()?,
                freq_div,
                self.loop_behavior(),
                segment,
                update_segment,
            ),
            Self::O2::default(),
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChangeModulationSegment {
    segment: Segment,
}

impl ChangeModulationSegment {
    pub const fn new(segment: Segment) -> Self {
        Self { segment }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl Datagram for ChangeModulationSegment {
    type O1 = crate::operation::ModulationChangeSegmentOp;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.segment), Self::O2::default()))
    }
}

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

    #[test]
    fn test_modulation_property() {
        let m = TestModulation {
            config: SamplingConfiguration::FREQ_4K_HZ,
            buf: vec![],
            loop_behavior: LoopBehavior::Infinite,
        };
        assert_eq!(SamplingConfiguration::FREQ_4K_HZ, m.sampling_config());
        assert_eq!(LoopBehavior::Infinite, m.loop_behavior());
    }

    #[test]
    fn test_modulation_len() -> anyhow::Result<()> {
        assert_eq!(
            TestModulation {
                config: SamplingConfiguration::FREQ_4K_HZ,
                buf: vec![],
                loop_behavior: LoopBehavior::Infinite,
            }
            .len()?,
            0
        );

        assert_eq!(
            TestModulation {
                config: SamplingConfiguration::FREQ_4K_HZ,
                buf: vec![EmitIntensity::MIN; 100],
                loop_behavior: LoopBehavior::Infinite,
            }
            .len()?,
            100
        );

        Ok(())
    }

    #[test]
    fn test_change_mod_segment() -> anyhow::Result<()> {
        let d = ChangeModulationSegment::new(Segment::S0);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(Duration::from_millis(200)), d.timeout());
        let _ = d.operation()?;
        Ok(())
    }
}
