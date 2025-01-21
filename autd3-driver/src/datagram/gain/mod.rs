mod boxed;

use autd3_core::gain::{Gain, GainContextGenerator, GainOperationGenerator};
pub use boxed::{BoxedGain, IntoBoxedGain};

use crate::{
    firmware::operation::{GainOp, NullOp, OperationGenerator},
    geometry::Device,
};

impl<G: GainContextGenerator> OperationGenerator for GainOperationGenerator<G> {
    type O1 = GainOp<G::Context>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        let context = self.generator.generate(device);
        (
            Self::O1::new(self.segment, self.transition, context),
            Self::O2 {},
        )
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use autd3_core::derive::*;

    use crate::firmware::fpga::{Drive, EmitIntensity, Phase};

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
                    .devices()
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

    pub struct Context {
        data: Vec<Drive>,
    }

    impl GainContext for Context {
        fn calc(&self, tr: &Transducer) -> Drive {
            self.data[tr.idx()]
        }
    }

    impl GainContextGenerator for TestGain {
        type Context = Context;

        fn generate(&mut self, device: &Device) -> Self::Context {
            Context {
                data: self.data.remove(&device.idx()).unwrap(),
            }
        }
    }

    impl Gain for TestGain {
        type G = Self;

        fn init(
            self,
            _geometry: &Geometry,
            _filter: Option<&HashMap<usize, BitVec>>,
            _option: &DatagramOption,
        ) -> Result<Self::G, GainError> {
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
        vec![true; 2],
        2)]
    #[case::parallel(
        [
            (0, vec![Drive { phase: Phase(0x01), intensity: EmitIntensity(0x01) }; NUM_TRANSDUCERS]),
            (1, vec![Drive { phase: Phase(0x02), intensity: EmitIntensity(0x02) }; NUM_TRANSDUCERS]),
            (2, vec![Drive { phase: Phase(0x03), intensity: EmitIntensity(0x03) }; NUM_TRANSDUCERS]),
            (3, vec![Drive { phase: Phase(0x04), intensity: EmitIntensity(0x04) }; NUM_TRANSDUCERS]),
            (4, vec![Drive { phase: Phase(0x05), intensity: EmitIntensity(0x05) }; NUM_TRANSDUCERS]),
        ].into_iter().collect(),
        vec![true; 5],
        5)]
    #[case::enabled(
        [
            (0, vec![Drive { phase: Phase(0x01), intensity: EmitIntensity(0x01) }; NUM_TRANSDUCERS]),
        ].into_iter().collect(),
        vec![true, false],
        2)]
    fn gain(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] enabled: Vec<bool>,
        #[case] n: u16,
    ) -> anyhow::Result<()> {
        use crate::datagram::tests::create_geometry;

        let mut geometry = create_geometry(n, NUM_TRANSDUCERS as _);
        geometry
            .iter_mut()
            .zip(enabled.iter())
            .for_each(|(dev, &e)| dev.enable = e);
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
        let mut f = g.init(&geometry, None, &DatagramOption::default())?;
        assert_eq!(
            expect,
            geometry
                .devices()
                .map(|dev| {
                    let f = f.generate(dev);
                    (dev.idx(), dev.iter().map(|tr| f.calc(tr)).collect())
                })
                .collect()
        );

        Ok(())
    }
}
