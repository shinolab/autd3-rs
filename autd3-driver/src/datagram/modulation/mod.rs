mod boxed;

use autd3_core::{derive::ModulationOperationGenerator, modulation::Modulation};
pub use boxed::{BoxedModulation, IntoBoxedModulation};

use super::silencer::HasSamplingConfig;
use crate::{
    firmware::{
        fpga::SamplingConfig,
        operation::{ModulationOp, NullOp, OperationGenerator},
    },
    geometry::Device,
};

impl<M: Modulation> HasSamplingConfig for M {
    fn intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config())
    }
    fn phase(&self) -> Option<SamplingConfig> {
        None
    }
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
            Self::O2 {},
        )
    }
}

#[cfg(test)]
pub mod tests {
    use autd3_core::{
        derive::LoopBehavior,
        modulation::{ModulationError, ModulationProperty},
    };

    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    pub struct TestModulation {
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl ModulationProperty for TestModulation {
        fn sampling_config(&self) -> SamplingConfig {
            self.config
        }
        fn loop_behavior(&self) -> LoopBehavior {
            self.loop_behavior
        }
    }

    impl Modulation for TestModulation {
        fn calc(self) -> Result<Vec<u8>, ModulationError> {
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
