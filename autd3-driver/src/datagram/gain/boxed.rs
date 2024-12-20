use std::{collections::HashMap, mem::MaybeUninit};

use super::{Gain, GainContextGenerator, GainOperationGenerator};

use crate::error::AUTDDriverError;
pub use crate::{
    datagram::DatagramS,
    firmware::{
        fpga::{Drive, Segment, TransitionMode},
        operation::GainContext,
    },
    geometry::{Device, Geometry, Transducer},
};

use autd3_derive::Gain;
use bit_vec::BitVec;

impl GainContext for Box<dyn GainContext> {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.as_ref().calc(tr)
    }
}

pub trait DGainContextGenerator {
    fn dyn_generate(&mut self, device: &Device) -> Box<dyn GainContext>;
}

pub struct DynGainContextGenerator {
    g: Box<dyn DGainContextGenerator>,
}

impl GainContextGenerator for DynGainContextGenerator {
    type Context = Box<dyn GainContext>;

    fn generate(&mut self, device: &Device) -> Box<dyn GainContext> {
        self.g.dyn_generate(device)
    }
}

impl<Context: GainContext + 'static, G: GainContextGenerator<Context = Context>>
    DGainContextGenerator for G
{
    fn dyn_generate(&mut self, device: &Device) -> Box<dyn GainContext> {
        Box::new(GainContextGenerator::generate(self, device))
    }
}

pub trait DGain {
    fn dyn_init(
        &mut self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Box<dyn DGainContextGenerator>, AUTDDriverError>;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<
        G: DGainContextGenerator + 'static,
        #[cfg(not(feature = "lightweight"))] T: Gain<G = G>,
        #[cfg(feature = "lightweight")] T: Gain<G = G> + Send + Sync,
    > DGain for MaybeUninit<T>
{
    fn dyn_init(
        &mut self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Box<dyn DGainContextGenerator>, AUTDDriverError> {
        let mut tmp: MaybeUninit<T> = MaybeUninit::uninit();
        std::mem::swap(&mut tmp, self);
        let g = unsafe { tmp.assume_init() };
        Ok(Box::new(g.init(geometry, filter)?) as _)
    }

    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { self.assume_init_ref() }.fmt(f)
    }
}

#[derive(Gain)]
pub struct BoxedGain {
    g: Box<dyn DGain>,
}

#[cfg(feature = "lightweight")]
unsafe impl Send for BoxedGain {}
#[cfg(feature = "lightweight")]
unsafe impl Sync for BoxedGain {}

impl std::fmt::Debug for BoxedGain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.g.as_ref().dyn_fmt(f)
    }
}

impl Gain for BoxedGain {
    type G = DynGainContextGenerator;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDDriverError> {
        let Self { mut g } = self;
        Ok(DynGainContextGenerator {
            g: g.dyn_init(geometry, filter)?,
        })
    }
}

pub trait IntoBoxedGain {
    fn into_boxed(self) -> BoxedGain;
}

impl<
        #[cfg(not(feature = "lightweight"))] G: Gain + 'static,
        #[cfg(feature = "lightweight")] G: Gain + Send + Sync + 'static,
    > IntoBoxedGain for G
{
    fn into_boxed(self) -> BoxedGain {
        BoxedGain {
            g: Box::new(MaybeUninit::new(self)),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::datagram::gain::tests::TestGain;

    use crate::firmware::fpga::{EmitIntensity, Phase};
    use crate::geometry::tests::create_geometry;

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
    fn boxed_gain(
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
        )
        .into_boxed();
        let mut f = g.init(&geometry, None)?;
        assert_eq!(
            expect,
            geometry
                .devices()
                .map(|dev| {
                    let f = GainContextGenerator::generate(&mut f, dev);
                    (dev.idx(), dev.iter().map(|tr| f.calc(tr)).collect())
                })
                .collect()
        );

        Ok(())
    }

    #[test]
    fn boxed_gain_dbg() {
        let g = TestGain::null();
        assert_eq!(format!("{:?}", g), format!("{:?}", g.into_boxed()));
    }
}
