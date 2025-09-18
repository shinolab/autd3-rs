mod boxed;

pub use boxed::BoxedModulation;

#[cfg(test)]
pub mod tests {
    use std::num::NonZeroU16;

    use autd3_core::derive::*;

    use crate::datagram::{
        with_loop_behavior::WithFiniteLoopInspectionResult,
        with_segment::WithSegmentInspectionResult,
    };

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
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(ModulationInspectionResult {
                    data: vec![0, 0],
                    config: SamplingConfig::FREQ_4K,
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
            transition_mode: transition_mode::Later,
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(WithSegmentInspectionResult {
                    inner: ModulationInspectionResult {
                        data: vec![0, 0],
                        config: SamplingConfig::FREQ_4K,
                    },
                    segment: Segment::S1,
                    transition_mode: transition_mode::Later,
                }),
                r
            );
        });

        Ok(())
    }

    #[test]
    fn inspect_with_loop_behavior() -> anyhow::Result<()> {
        let geometry = crate::autd3_device::tests::create_geometry(2);

        crate::datagram::WithFiniteLoop {
            inner: TestModulation {
                sampling_config: SamplingConfig::FREQ_4K,
            },
            segment: Segment::S1,
            transition_mode: transition_mode::SyncIdx,
            loop_count: NonZeroU16::MIN,
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(WithFiniteLoopInspectionResult {
                    inner: ModulationInspectionResult {
                        data: vec![0, 0],
                        config: SamplingConfig::FREQ_4K,
                    },
                    segment: Segment::S1,
                    transition_mode: transition_mode::SyncIdx,
                    loop_count: NonZeroU16::MIN,
                }),
                r
            );
        });

        Ok(())
    }
}
