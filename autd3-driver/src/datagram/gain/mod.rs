mod boxed;

use autd3_core::gain::{Gain, GainCalculatorGenerator};
pub use boxed::BoxedGain;

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use autd3_core::{
        derive::*,
        geometry::{Point3, UnitQuaternion},
    };

    use crate::datagram::with_segment::WithSegmentInspectionResult;

    #[derive(Gain, Clone, Debug)]
    pub struct TestGain {
        pub data: HashMap<usize, Vec<Drive>>,
    }

    impl TestGain {
        pub fn new<'a, FT: Fn(&'a Transducer) -> Drive, F: Fn(&'a Device) -> FT>(
            f: F,
            geometry: &'a Geometry,
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

    impl GainCalculator<'_> for Impl {
        fn calc(&self, tr: &Transducer) -> Drive {
            self.data[tr.idx()]
        }
    }

    impl GainCalculatorGenerator<'_> for TestGain {
        type Calculator = Impl;

        fn generate(&mut self, device: &Device) -> Self::Calculator {
            Impl {
                data: self.data.remove(&device.idx()).unwrap(),
            }
        }
    }

    impl Gain<'_> for TestGain {
        type G = Self;

        fn init(
            self,
            _: &Geometry,
            _env: &Environment,
            _: &TransducerMask,
        ) -> Result<Self::G, GainError> {
            Ok(self)
        }
    }

    pub fn create_geometry(num_dev: usize, num_trans: usize) -> Geometry {
        Geometry::new(
            (0..num_dev)
                .map(|_| {
                    Device::new(
                        UnitQuaternion::identity(),
                        (0..num_trans)
                            .map(|_| Transducer::new(Point3::origin()))
                            .collect(),
                    )
                })
                .collect(),
        )
    }

    const NUM_TRANSDUCERS: usize = 2;

    #[rstest::rstest]
    #[case::serial(
        [
            (0, vec![Drive { phase: Phase(0x01), intensity: Intensity(0x01) }; NUM_TRANSDUCERS]),
            (1, vec![Drive { phase: Phase(0x02), intensity: Intensity(0x02) }; NUM_TRANSDUCERS])
        ].into_iter().collect(),
        2)]
    #[case::parallel(
        [
            (0, vec![Drive { phase: Phase(0x01), intensity: Intensity(0x01) }; NUM_TRANSDUCERS]),
            (1, vec![Drive { phase: Phase(0x02), intensity: Intensity(0x02) }; NUM_TRANSDUCERS]),
            (2, vec![Drive { phase: Phase(0x03), intensity: Intensity(0x03) }; NUM_TRANSDUCERS]),
            (3, vec![Drive { phase: Phase(0x04), intensity: Intensity(0x04) }; NUM_TRANSDUCERS]),
            (4, vec![Drive { phase: Phase(0x05), intensity: Intensity(0x05) }; NUM_TRANSDUCERS]),
        ].into_iter().collect(),
        5)]
    fn gain(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] n: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let geometry = create_geometry(n, NUM_TRANSDUCERS);

        let g = TestGain::new(
            |dev| {
                move |_| Drive {
                    phase: Phase(dev.idx() as u8 + 1),
                    intensity: Intensity(dev.idx() as u8 + 1),
                }
            },
            &geometry,
        );
        let mut f = g.init(&geometry, &Environment::new(), &TransducerMask::AllEnabled)?;
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
    fn inspect() -> Result<(), Box<dyn std::error::Error>> {
        let geometry = create_geometry(2, 1);

        TestGain::new(
            |_dev| {
                |_| Drive {
                    phase: Phase(0xFF),
                    intensity: Intensity(0xFF),
                }
            },
            &geometry,
        )
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(GainInspectionResult {
                    data: vec![
                        Drive {
                            phase: Phase(0xFF),
                            intensity: Intensity(0xFF),
                        };
                        1
                    ],
                }),
                r
            );
        });

        Ok(())
    }

    #[test]
    fn inspect_with_segment() -> Result<(), Box<dyn std::error::Error>> {
        let geometry = create_geometry(2, 1);

        crate::datagram::WithSegment {
            inner: TestGain::new(
                |_dev| {
                    |_| Drive {
                        phase: Phase(0xFF),
                        intensity: Intensity(0xFF),
                    }
                },
                &geometry,
            ),
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
                    inner: GainInspectionResult {
                        data: vec![
                            Drive {
                                phase: Phase(0xFF),
                                intensity: Intensity(0xFF),
                            };
                            1
                        ],
                    },
                    segment: Segment::S1,
                    transition_mode: transition_mode::Later,
                }),
                r
            );
        });

        Ok(())
    }
}
