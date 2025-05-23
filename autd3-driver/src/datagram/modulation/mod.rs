mod boxed;

use autd3_core::derive::{ModulationInspectionResult, ModulationOperationGenerator};
pub use boxed::{BoxedModulation, IntoBoxedModulation};

use crate::{
    firmware::operation::{ModulationOp, NullOp, OperationGenerator},
    geometry::Device,
};

use super::{
    with_loop_behavior::InspectionResultWithLoopBehavior, with_segment::InspectionResultWithSegment,
};

impl OperationGenerator for ModulationOperationGenerator {
    type O1 = ModulationOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        let d = self.g.clone();
        Some((
            Self::O1::new(
                d,
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2 {},
        ))
    }
}

impl InspectionResultWithSegment for ModulationInspectionResult {
    fn with_segment(
        self,
        segment: autd3_core::derive::Segment,
        transition_mode: Option<autd3_core::derive::TransitionMode>,
    ) -> Self {
        Self {
            segment,
            transition_mode,
            ..self
        }
    }
}

impl InspectionResultWithLoopBehavior for ModulationInspectionResult {
    fn with_loop_behavior(
        self,
        loop_behavior: autd3_core::derive::LoopBehavior,
        segment: autd3_core::derive::Segment,
        transition_mode: Option<autd3_core::derive::TransitionMode>,
    ) -> Self {
        Self {
            loop_behavior,
            segment,
            transition_mode,
            ..self
        }
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

        fn sampling_config(&self) -> SamplingConfig {
            self.sampling_config
        }
    }
}
