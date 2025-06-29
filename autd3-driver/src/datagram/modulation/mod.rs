mod boxed;

pub use boxed::BoxedModulation;

use super::{
    with_loop_behavior::InspectionResultWithLoopBehavior, with_segment::InspectionResultWithSegment,
};

use autd3_core::{
    datagram::{LoopBehavior, Segment, TransitionMode},
    modulation::ModulationInspectionResult,
};

impl InspectionResultWithSegment for ModulationInspectionResult {
    fn with_segment(self, segment: Segment, transition_mode: Option<TransitionMode>) -> Self {
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
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
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
        fn calc(self, _: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
            Ok(vec![0; 2])
        }

        fn sampling_config(&self) -> SamplingConfig {
            self.sampling_config
        }
    }

    #[test]
    fn inspect() -> anyhow::Result<()> {
        let geometry = crate::autd3_device::tests::create_geometry(2);

        TestModulation {
            sampling_config: SamplingConfig::FREQ_4K,
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(ModulationInspectionResult {
                    name: "TestModulation".to_string(),
                    data: vec![0, 0],
                    config: SamplingConfig::FREQ_4K,
                    loop_behavior: LoopBehavior::Infinite,
                    segment: Segment::S0,
                    transition_mode: None,
                }),
                r
            );
        });

        Ok(())
    }

    #[test]
    fn inspect_with_segment() -> anyhow::Result<()> {
        let geometry = crate::autd3_device::tests::create_geometry(2);

        crate::datagram::WithSegment {
            inner: TestModulation {
                sampling_config: SamplingConfig::FREQ_4K,
            },
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(ModulationInspectionResult {
                    name: "TestModulation".to_string(),
                    data: vec![0, 0],
                    config: SamplingConfig::FREQ_4K,
                    loop_behavior: LoopBehavior::Infinite,
                    segment: Segment::S1,
                    transition_mode: Some(TransitionMode::Immediate),
                }),
                r
            );
        });

        Ok(())
    }

    #[test]
    fn inspect_with_loop_behavior() -> anyhow::Result<()> {
        let geometry = crate::autd3_device::tests::create_geometry(2);

        crate::datagram::WithLoopBehavior {
            inner: TestModulation {
                sampling_config: SamplingConfig::FREQ_4K,
            },
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
            loop_behavior: LoopBehavior::ONCE,
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(ModulationInspectionResult {
                    name: "TestModulation".to_string(),
                    data: vec![0, 0],
                    config: SamplingConfig::FREQ_4K,
                    loop_behavior: LoopBehavior::ONCE,
                    segment: Segment::S1,
                    transition_mode: Some(TransitionMode::Immediate),
                }),
                r
            );
        });

        Ok(())
    }
}
