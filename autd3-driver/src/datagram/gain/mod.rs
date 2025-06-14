mod boxed;

use autd3_core::gain::{
    Gain, GainCalculatorGenerator, GainInspectionResult, GainOperationGenerator,
};
pub use boxed::BoxedGain;

use crate::{
    firmware::operation::{GainOp, NullOp, OperationGenerator},
    geometry::Device,
};

use super::with_segment::InspectionResultWithSegment;

impl<G: GainCalculatorGenerator> OperationGenerator for GainOperationGenerator<G> {
    type O1 = GainOp<G::Calculator>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        let c = self.generator.generate(device);
        Some((Self::O1::new(self.segment, self.transition, c), Self::O2 {}))
    }
}

impl InspectionResultWithSegment for GainInspectionResult {
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

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use autd3_core::derive::*;

    use crate::{
        datagram::tests::create_geometry,
        firmware::fpga::{Drive, EmitIntensity, Phase},
    };

    #[derive(Gain, Clone, Debug)]
    pub struct TestGain {
        pub data: HashMap<usize, Vec<Drive>>,
    }

    impl TestGain {
        pub fn new<FT: Fn(&Transducer) -> Drive, F: Fn(&Device) -> FT>(
            f: F,
            geometry: &Geometry,
        ) -> Self {
            Self {
                data: geometry
                    .iter()
                    .map(|dev| (dev.idx(), dev.iter().map(f(dev)).collect()))
                    .collect(),
            }
        }

        pub fn null() -> Self {
            Self {
                data: HashMap::new(),
            }
        }
    }

    pub struct Impl {
        data: Vec<Drive>,
    }

    impl GainCalculator for Impl {
        fn calc(&self, tr: &Transducer) -> Drive {
            self.data[tr.idx()]
        }
    }

    impl GainCalculatorGenerator for TestGain {
        type Calculator = Impl;

        fn generate(&mut self, device: &Device) -> Self::Calculator {
            Impl {
                data: self.data.remove(&device.idx()).unwrap(),
            }
        }
    }

    impl Gain for TestGain {
        type G = Self;

        fn init(self, _: &Geometry, _: &TransducerFilter) -> Result<Self::G, GainError> {
            Ok(self)
        }
    }

    const NUM_TRANSDUCERS: usize = 2;

    #[rstest::rstest]
    #[test]
    #[case::serial(
        [
            (0, vec![Drive { phase: Phase(0x01), intensity: EmitIntensity(0x01) }; NUM_TRANSDUCERS]),
            (1, vec![Drive { phase: Phase(0x02), intensity: EmitIntensity(0x02) }; NUM_TRANSDUCERS])
        ].into_iter().collect(),
        2)]
    #[case::parallel(
        [
            (0, vec![Drive { phase: Phase(0x01), intensity: EmitIntensity(0x01) }; NUM_TRANSDUCERS]),
            (1, vec![Drive { phase: Phase(0x02), intensity: EmitIntensity(0x02) }; NUM_TRANSDUCERS]),
            (2, vec![Drive { phase: Phase(0x03), intensity: EmitIntensity(0x03) }; NUM_TRANSDUCERS]),
            (3, vec![Drive { phase: Phase(0x04), intensity: EmitIntensity(0x04) }; NUM_TRANSDUCERS]),
            (4, vec![Drive { phase: Phase(0x05), intensity: EmitIntensity(0x05) }; NUM_TRANSDUCERS]),
        ].into_iter().collect(),
        5)]
    fn gain(#[case] expect: HashMap<usize, Vec<Drive>>, #[case] n: u16) -> anyhow::Result<()> {
        use crate::datagram::tests::create_geometry;

        let geometry = create_geometry(n, NUM_TRANSDUCERS as _);
        let g = TestGain::new(
            |dev| {
                let dev_idx = dev.idx();
                move |_| Drive {
                    phase: Phase(dev_idx as u8 + 1),
                    intensity: EmitIntensity(dev_idx as u8 + 1),
                }
            },
            &geometry,
        );
        let mut f = g.init(&geometry, &TransducerFilter::all_enabled())?;
        assert_eq!(
            expect,
            geometry
                .iter()
                .map(|dev| {
                    let f = f.generate(dev);
                    (dev.idx(), dev.iter().map(|tr| f.calc(tr)).collect())
                })
                .collect()
        );

        Ok(())
    }

    #[test]
    fn inspect() -> anyhow::Result<()> {
        let geometry = create_geometry(2, 1);

        TestGain::new(
            |_dev| {
                |_| Drive {
                    phase: Phase(0xFF),
                    intensity: EmitIntensity(0xFF),
                }
            },
            &geometry,
        )
        .inspect(&geometry, &DeviceFilter::all_enabled())?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(GainInspectionResult {
                    name: "TestGain".to_string(),
                    data: vec![
                        Drive {
                            phase: Phase(0xFF),
                            intensity: EmitIntensity(0xFF),
                        };
                        1
                    ],
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
        let geometry = create_geometry(2, 1);

        crate::datagram::WithSegment {
            inner: TestGain::new(
                |_dev| {
                    |_| Drive {
                        phase: Phase(0xFF),
                        intensity: EmitIntensity(0xFF),
                    }
                },
                &geometry,
            ),
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
        }
        .inspect(&geometry, &DeviceFilter::all_enabled())?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(GainInspectionResult {
                    name: "TestGain".to_string(),
                    data: vec![
                        Drive {
                            phase: Phase(0xFF),
                            intensity: EmitIntensity(0xFF),
                        };
                        1
                    ],
                    segment: Segment::S1,
                    transition_mode: Some(TransitionMode::Immediate),
                }),
                r
            );
        });

        Ok(())
    }
}
