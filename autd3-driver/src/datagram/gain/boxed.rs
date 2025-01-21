use std::{collections::HashMap, mem::MaybeUninit};

use super::{Gain, GainContextGenerator};

pub use crate::geometry::{Device, Geometry};

use autd3_core::derive::*;

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

/// A dyn-compatible version of [`Gain`].
trait DGain {
    fn dyn_init(
        &mut self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
        option: &DatagramOption,
    ) -> Result<Box<dyn DGainContextGenerator>, GainError>;
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
        filter: Option<&HashMap<usize, BitVec>>,
        option: &DatagramOption,
    ) -> Result<Box<dyn DGainContextGenerator>, GainError> {
        let mut tmp: MaybeUninit<T> = MaybeUninit::uninit();
        std::mem::swap(&mut tmp, self);
        // SAFETY: This function is called only once from `Gain::init`.
        let g = unsafe { tmp.assume_init() };
        Ok(Box::new(g.init(geometry, filter, option)?) as _)
    }

    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: This function is never called after `dyn_init`.
        unsafe { self.assume_init_ref() }.fmt(f)
    }
}

/// Boxed [`Gain`].
///
/// Because [`Gain`] traits can have different associated types, it cannot simply be wrapped in a [`Box`] like `Box<dyn Gain>`.
/// [`BoxedGain`] provides the ability to wrap any [`Gain`] in a common type.
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
        filter: Option<&HashMap<usize, BitVec>>,
        option: &DatagramOption,
    ) -> Result<Self::G, GainError> {
        let Self { mut g, .. } = self;
        Ok(DynGainContextGenerator {
            g: g.dyn_init(geometry, filter, option)?,
        })
    }
}

/// Trait to convert [`Gain`] to [`BoxedGain`].
pub trait IntoBoxedGain {
    /// Convert [`Gain`] to [`BoxedGain`].
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
    use autd3_core::gain::Drive;

    use super::*;
    use crate::datagram::gain::tests::TestGain;

    use crate::firmware::fpga::{EmitIntensity, Phase};

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
    fn boxed_gain(
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
        )
        .into_boxed();

        let mut f = g.init(&geometry, None, &DatagramOption::default())?;
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
