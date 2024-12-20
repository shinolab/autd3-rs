mod boxed;

pub use boxed::{BoxedModulation, IntoBoxedModulation};

use std::sync::Arc;

use super::silencer::WithSampling;
use crate::{
    error::AUTDDriverError,
    firmware::{
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::{ModulationOp, NullOp, OperationGenerator},
    },
    geometry::Device,
};

pub trait ModulationProperty {
    fn sampling_config(&self) -> SamplingConfig;
    fn loop_behavior(&self) -> LoopBehavior;
}

pub trait Modulation: ModulationProperty + std::fmt::Debug {
    fn calc(self) -> Result<Vec<u8>, AUTDDriverError>;
}

impl<M: Modulation> WithSampling for M {
    fn sampling_config_intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config())
    }
    fn sampling_config_phase(&self) -> Option<SamplingConfig> {
        None
    }
}

pub struct ModulationOperationGenerator {
    pub g: Arc<Vec<u8>>,
    pub config: SamplingConfig,
    pub loop_behavior: LoopBehavior,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}

impl OperationGenerator for ModulationOperationGenerator {
    type O1 = ModulationOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        let d = self.g.clone();
        (
            Self::O1::new(
                d,
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2::new(),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use autd3_derive::Modulation;

    use super::*;
    use crate::{datagram::DatagramS, geometry::Geometry};

    #[derive(Modulation, Clone, PartialEq, Debug)]
    pub struct TestModulation {
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl Modulation for TestModulation {
        fn calc(self) -> Result<Vec<u8>, AUTDDriverError> {
            Ok(vec![0; 2])
        }
    }

    #[test]
    fn test() {
        let m = TestModulation {
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        };

        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(LoopBehavior::infinite(), m.loop_behavior());
        assert_eq!(Ok(vec![0; 2]), m.calc());
    }
}
