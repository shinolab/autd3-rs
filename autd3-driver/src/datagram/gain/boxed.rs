use std::mem::MaybeUninit;

use super::{Gain, GainCalculatorGenerator};

pub use crate::geometry::{Device, Geometry};

use autd3_core::{derive::*, gain::TransducerFilter};

#[cfg(not(feature = "lightweight"))]
pub trait DGainCalculatorGenerator {
    #[must_use]
    fn dyn_generate(&mut self, device: &Device) -> Box<dyn GainCalculator>;
}
#[cfg(feature = "lightweight")]
pub trait DGainCalculatorGenerator: Send + Sync {
    #[must_use]
    fn dyn_generate(&mut self, device: &Device) -> Box<dyn GainCalculator>;
}

pub struct DynGainCalculatorGenerator {
    g: Box<dyn DGainCalculatorGenerator>,
}

impl GainCalculatorGenerator for DynGainCalculatorGenerator {
    type Calculator = Box<dyn GainCalculator>;

    fn generate(&mut self, device: &Device) -> Box<dyn GainCalculator> {
        self.g.dyn_generate(device)
    }
}

impl<
    Calculator: GainCalculator + 'static,
    #[cfg(not(feature = "lightweight"))] G: GainCalculatorGenerator<Calculator = Calculator>,
    #[cfg(feature = "lightweight")] G: GainCalculatorGenerator<Calculator = Calculator> + Send + Sync,
> DGainCalculatorGenerator for G
{
    fn dyn_generate(&mut self, device: &Device) -> Box<dyn GainCalculator> {
        Box::new(GainCalculatorGenerator::generate(self, device))
    }
}

/// A dyn-compatible version of [`Gain`].
trait DGain {
    fn dyn_init(
        &mut self,
        geometry: &Geometry,
        filter: &TransducerFilter,
    ) -> Result<Box<dyn DGainCalculatorGenerator>, GainError>;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<
    G: DGainCalculatorGenerator + 'static,
    #[cfg(not(feature = "lightweight"))] T: Gain<G = G>,
    #[cfg(feature = "lightweight")] T: Gain<G = G> + Send + Sync,
> DGain for MaybeUninit<T>
{
    fn dyn_init(
        &mut self,
        geometry: &Geometry,
        filter: &TransducerFilter,
    ) -> Result<Box<dyn DGainCalculatorGenerator>, GainError> {
        let mut tmp: MaybeUninit<T> = MaybeUninit::uninit();
        std::mem::swap(&mut tmp, self);
        // SAFETY: This function is called only once from `Gain::init`.
        let g = unsafe { tmp.assume_init() };
        Ok(Box::new(g.init(geometry, filter)?) as _)
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

impl BoxedGain {
    /// Creates a new [`BoxedGain`].
    #[must_use]
    pub fn new<
        #[cfg(feature = "lightweight")] GG: GainCalculatorGenerator + Send + Sync + 'static,
        #[cfg(not(feature = "lightweight"))] G: Gain + 'static,
        #[cfg(feature = "lightweight")] G: Gain<G = GG> + Send + Sync + 'static,
    >(
        g: G,
    ) -> Self {
        Self {
            g: Box::new(MaybeUninit::new(g)),
        }
    }
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
    type G = DynGainCalculatorGenerator;

    fn init(self, geometry: &Geometry, filter: &TransducerFilter) -> Result<Self::G, GainError> {
        let Self { mut g, .. } = self;
        Ok(DynGainCalculatorGenerator {
            g: g.dyn_init(geometry, filter)?,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::datagram::gain::tests::TestGain;

    use autd3_core::{
        gain::Drive,
        geometry::{Point3, UnitQuaternion},
    };

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
    fn boxed_gain_unsafe(
        #[case] expect: HashMap<usize, Vec<Drive>>,
        #[case] n: u16,
    ) -> anyhow::Result<()> {
        let geometry = Geometry::new(
            (0..n)
                .map(|_| {
                    Device::new(
                        UnitQuaternion::identity(),
                        (0..NUM_TRANSDUCERS)
                            .map(|_| Transducer::new(Point3::origin()))
                            .collect(),
                    )
                })
                .collect(),
        );

        let g = BoxedGain::new(TestGain::new(
            |dev| {
                let dev_idx = dev.idx();
                move |_| Drive {
                    phase: Phase(dev_idx as u8 + 1),
                    intensity: EmitIntensity(dev_idx as u8 + 1),
                }
            },
            &geometry,
        ));

        let mut f = g.init(&geometry, &TransducerFilter::all_enabled())?;
        assert_eq!(
            expect,
            geometry
                .iter()
                .map(|dev| {
                    let f = GainCalculatorGenerator::generate(&mut f, dev);
                    (dev.idx(), dev.iter().map(|tr| f.calc(tr)).collect())
                })
                .collect()
        );

        Ok(())
    }

    #[test]
    fn boxed_gain_dbg_unsafe() {
        let g = TestGain::null();
        assert_eq!(format!("{:?}", g), format!("{:?}", BoxedGain::new(g)));
    }
}
