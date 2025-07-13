use std::mem::MaybeUninit;

use super::{Gain, GainCalculatorGenerator};

pub use crate::geometry::{Device, Geometry};

use autd3_core::{derive::*, gain::TransducerFilter};

pub trait DGainCalculatorGenerator<'dev, 'tr>
where
    'dev: 'tr,
{
    #[must_use]
    fn dyn_generate(&mut self, device: &'dev Device) -> Box<dyn GainCalculator<'tr>>;
}

pub struct DynGainCalculatorGenerator<'dev, 'tr> {
    g: Box<dyn DGainCalculatorGenerator<'dev, 'tr>>,
}

impl<'dev, 'tr> GainCalculatorGenerator<'dev, 'tr> for DynGainCalculatorGenerator<'dev, 'tr>
where
    'dev: 'tr,
{
    type Calculator = Box<dyn GainCalculator<'tr>>;

    fn generate(&mut self, device: &'dev Device) -> Box<dyn GainCalculator<'tr>> {
        self.g.dyn_generate(device)
    }
}

impl<
    'dev,
    'tr,
    Calculator: GainCalculator<'tr> + 'static,
    G: GainCalculatorGenerator<'dev, 'tr, Calculator = Calculator>,
> DGainCalculatorGenerator<'dev, 'tr> for G
where
    'dev: 'tr,
{
    fn dyn_generate(&mut self, device: &'dev Device) -> Box<dyn GainCalculator<'tr>> {
        Box::new(GainCalculatorGenerator::generate(self, device))
    }
}

/// A dyn-compatible version of [`Gain`].
trait DGain<'geo, 'dev, 'tr> {
    fn dyn_init(
        &mut self,
        geometry: &'geo Geometry,
        env: &Environment,
        filter: &TransducerFilter,
    ) -> Result<Box<dyn DGainCalculatorGenerator<'dev, 'tr>>, GainError>;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<
    'geo,
    'dev,
    'tr,
    G: DGainCalculatorGenerator<'dev, 'tr> + 'static,
    T: Gain<'geo, 'dev, 'tr, G = G>,
> DGain<'geo, 'dev, 'tr> for MaybeUninit<T>
where
    'geo: 'dev,
    'dev: 'tr,
{
    fn dyn_init(
        &mut self,
        geometry: &'geo Geometry,
        env: &Environment,
        filter: &TransducerFilter,
    ) -> Result<Box<dyn DGainCalculatorGenerator<'dev, 'tr>>, GainError> {
        let mut tmp: MaybeUninit<T> = MaybeUninit::uninit();
        std::mem::swap(&mut tmp, self);
        // SAFETY: This function is called only once from `Gain::init`.
        let g = unsafe { tmp.assume_init() };
        Ok(Box::new(g.init(geometry, env, filter)?) as _)
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
pub struct BoxedGain<'geo, 'dev, 'tr> {
    g: Box<dyn DGain<'geo, 'dev, 'tr>>,
}

impl<'geo, 'dev, 'tr> BoxedGain<'geo, 'dev, 'tr>
where
    'geo: 'dev,
    'dev: 'tr,
{
    /// Creates a new [`BoxedGain`].
    #[must_use]
    pub fn new<
        C: GainCalculator<'tr> + 'static,
        GG: GainCalculatorGenerator<'dev, 'tr, Calculator = C> + 'static,
        G: Gain<'geo, 'dev, 'tr, G = GG> + 'static,
    >(
        g: G,
    ) -> Self {
        Self {
            g: Box::new(MaybeUninit::new(g)),
        }
    }
}

impl std::fmt::Debug for BoxedGain<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.g.as_ref().dyn_fmt(f)
    }
}

impl<'geo, 'dev, 'tr> Gain<'geo, 'dev, 'tr> for BoxedGain<'geo, 'dev, 'tr>
where
    'geo: 'dev,
    'dev: 'tr,
{
    type G = DynGainCalculatorGenerator<'dev, 'tr>;

    fn init(
        self,
        geometry: &'geo Geometry,
        env: &Environment,
        filter: &TransducerFilter,
    ) -> Result<Self::G, GainError> {
        let Self { mut g, .. } = self;
        Ok(DynGainCalculatorGenerator {
            g: g.dyn_init(geometry, env, filter)?,
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
                move |_| Drive {
                    phase: Phase(dev.idx() as u8 + 1),
                    intensity: Intensity(dev.idx() as u8 + 1),
                }
            },
            &geometry,
        ));

        let mut f = g.init(
            &geometry,
            &Environment::new(),
            &TransducerFilter::all_enabled(),
        )?;
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
        assert_eq!(format!("{g:?}"), format!("{:?}", BoxedGain::new(g)));
    }
}
