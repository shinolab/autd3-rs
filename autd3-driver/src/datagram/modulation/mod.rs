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

    use crate::datagram::tests::create_geometry;

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

    #[test]
    fn inspect() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);

        geometry[1].enable = false;

        let r = TestModulation {
            sampling_config: SamplingConfig::FREQ_4K,
        }
        .inspect(&mut geometry)?;

        assert_eq!(
            Some(ModulationInspectionResult {
                name: "TestModulation".to_string(),
                data: vec![0, 0],
                config: SamplingConfig::FREQ_4K,
                loop_behavior: LoopBehavior::Infinite,
                segment: Segment::S0,
                transition_mode: None,
            }),
            r[0]
        );
        assert_eq!(None, r[1]);

        Ok(())
    }

    #[test]
    fn inspect_with_segment() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);

        geometry[1].enable = false;

        let r = crate::datagram::WithSegment {
            inner: TestModulation {
                sampling_config: SamplingConfig::FREQ_4K,
            },
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
        }
        .inspect(&mut geometry)?;

        assert_eq!(
            Some(ModulationInspectionResult {
                name: "TestModulation".to_string(),
                data: vec![0, 0],
                config: SamplingConfig::FREQ_4K,
                loop_behavior: LoopBehavior::Infinite,
                segment: Segment::S1,
                transition_mode: Some(TransitionMode::Immediate),
            }),
            r[0]
        );
        assert_eq!(None, r[1]);

        Ok(())
    }

    #[test]
    fn inspect_with_loop_behavior() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);

        geometry[1].enable = false;

        let r = crate::datagram::WithLoopBehavior {
            inner: TestModulation {
                sampling_config: SamplingConfig::FREQ_4K,
            },
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
            loop_behavior: LoopBehavior::ONCE,
        }
        .inspect(&mut geometry)?;

        assert_eq!(
            Some(ModulationInspectionResult {
                name: "TestModulation".to_string(),
                data: vec![0, 0],
                config: SamplingConfig::FREQ_4K,
                loop_behavior: LoopBehavior::ONCE,
                segment: Segment::S1,
                transition_mode: Some(TransitionMode::Immediate),
            }),
            r[0]
        );
        assert_eq!(None, r[1]);

        Ok(())
    }
}
