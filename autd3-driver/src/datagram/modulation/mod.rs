mod boxed;

use autd3_core::derive::ModulationOperationGenerator;
pub use boxed::{BoxedModulation, IntoBoxedModulation};

use crate::{
    firmware::operation::{ModulationOp, NullOp, OperationGenerator},
    geometry::Device,
};

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
    use autd3_core::derive::*;

    #[derive(Modulation, Clone, PartialEq, Debug)]
    pub struct TestModulation {
        pub sampling_config: SamplingConfig,
    }

    impl Modulation for TestModulation {
        fn calc(self) -> Result<Vec<u8>, ModulationError> {
            Ok(vec![0; 2])
        }

        fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
            Ok(self.sampling_config)
        }
    }
}
