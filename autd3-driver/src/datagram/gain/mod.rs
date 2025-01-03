mod boxed;

pub use boxed::{BoxedGain, IntoBoxedGain};

use std::collections::HashMap;

pub use crate::firmware::operation::GainContext;
use crate::{
    error::AUTDDriverError,
    firmware::fpga::{Segment, TransitionMode},
    firmware::operation::{GainOp, NullOp, OperationGenerator},
    geometry::{Device, Geometry},
};

use bit_vec::BitVec;

/// A trait for generating a context for the gain operation.
pub trait GainContextGenerator {
    /// The type of the context that actually performs the calculation.
    type Context: GainContext;

    /// Generate a context for the given device.
    fn generate(&mut self, device: &Device) -> Self::Context;
}

/// Trait for calculating the phase/amplitude of each transducer.
///
/// See also [`Gain`] derive macro.
///
/// [`Gain`]: autd3_derive::Gain
pub trait Gain: std::fmt::Debug {
    /// The type of the context generator.
    type G: GainContextGenerator;

    /// Initialize the gain and generate the context generator.
    ///
    /// `filter` is a hash map that holds a bit vector representing the indices of the enabled transducers for each device index.
    /// If `filter` is `None`, all transducers are enabled.
    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDDriverError>;
}

#[doc(hidden)]
pub struct GainOperationGenerator<G: GainContextGenerator> {
    pub generator: G,
    pub segment: Segment,
    pub transition: Option<TransitionMode>,
}

impl<G: GainContextGenerator> GainOperationGenerator<G> {
    pub fn new<T: Gain<G = G>>(
        gain: T,
        geometry: &Geometry,
        segment: Segment,
        transition: Option<TransitionMode>,
    ) -> Result<Self, AUTDDriverError> {
        Ok(Self {
            generator: gain.init(geometry, None)?,
            segment,
            transition,
        })
    }
}

impl<G: GainContextGenerator> OperationGenerator for GainOperationGenerator<G> {
    type O1 = GainOp<G::Context>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        let context = self.generator.generate(device);
        (
            Self::O1::new(self.segment, self.transition, context),
            Self::O2::new(),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use autd3_derive::Gain;

    use super::*;

    use crate::{
        datagram::DatagramS,
        firmware::fpga::{Drive, EmitIntensity, Phase},
        geometry::{tests::create_geometry, Device, Transducer},
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
            _filter: Option<&HashMap<usize, BitVec<u32>>>,
        ) -> Result<Self::G, AUTDDriverError> {
            Ok(self)
        }
    }

    const NUM_TRANSDUCERS: usize = 2;

    #[rstest::rstest]
    #[test]
    #[case::serial(
        [
            (0, vec![Drive::new(Phase::new(0x01), EmitIntensity::new(0x01)); NUM_TRANSDUCERS]),
            (1, vec![Drive::new(Phase::new(0x02), EmitIntensity::new(0x02)); NUM_TRANSDUCERS])
        ].into_iter().collect(),
        vec![true; 2],
        2)]
    #[case::parallel(
        [
            (0, vec![Drive::new(Phase::new(0x01), EmitIntensity::new(0x01)); NUM_TRANSDUCERS]),
            (1, vec![Drive::new(Phase::new(0x02), EmitIntensity::new(0x02)); NUM_TRANSDUCERS]),
            (2, vec![Drive::new(Phase::new(0x03), EmitIntensity::new(0x03)); NUM_TRANSDUCERS]),
            (3, vec![Drive::new(Phase::new(0x04), EmitIntensity::new(0x04)); NUM_TRANSDUCERS]),
            (4, vec![Drive::new(Phase::new(0x05), EmitIntensity::new(0x05)); NUM_TRANSDUCERS]),
        ].into_iter().collect(),
        vec![true; 5],
        5)]
    #[case::enabled(
        [
            (0, vec![Drive::new(Phase::new(0x01), EmitIntensity::new(0x01)); NUM_TRANSDUCERS]),
        ].into_iter().collect(),
        vec![true, false],
        2)]
    fn gain(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] enabled: Vec<bool>,
        #[case] n: u16,
    ) -> anyhow::Result<()> {
        let mut geometry = create_geometry(n, NUM_TRANSDUCERS as _);
        geometry
            .iter_mut()
            .zip(enabled.iter())
            .for_each(|(dev, &e)| dev.enable = e);
        let g = TestGain::new(
            |dev| {
                let dev_idx = dev.idx();
                move |_| {
                    Drive::new(
                        Phase::new(dev_idx as u8 + 1),
                        EmitIntensity::new(dev_idx as u8 + 1),
                    )
                }
            },
            &geometry,
        );
        let mut f = g.init(&geometry, None)?;
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
