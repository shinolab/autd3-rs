use std::mem::MaybeUninit;

use super::{Gain, GainCalculatorGenerator};

pub use crate::geometry::{Device, Geometry};

use autd3_core::{derive::*, gain::TransducerFilter};

pub trait DGainCalculatorGenerator<'a> {
    #[must_use]
    fn dyn_generate(&mut self, device: &'a Device) -> Box<dyn GainCalculator<'a>>;
}

pub struct DynGainCalculatorGenerator<'a> {
    g: Box<dyn DGainCalculatorGenerator<'a>>,
}

impl<'a> GainCalculatorGenerator<'a> for DynGainCalculatorGenerator<'a> {
    type Calculator = Box<dyn GainCalculator<'a>>;

    fn generate(&mut self, device: &'a Device) -> Box<dyn GainCalculator<'a>> {
        self.g.dyn_generate(device)
    }
}

impl<
    'a,
    Calculator: GainCalculator<'a> + 'static,
    G: GainCalculatorGenerator<'a, Calculator = Calculator>,
> DGainCalculatorGenerator<'a> for G
{
    fn dyn_generate(&mut self, device: &'a Device) -> Box<dyn GainCalculator<'a>> {
        Box::new(GainCalculatorGenerator::generate(self, device))
    }
}

/// A dyn-compatible version of [`Gain`].
trait DGain<'a> {
    fn dyn_init(
        &mut self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &TransducerFilter,
    ) -> Result<Box<dyn DGainCalculatorGenerator<'a>>, GainError>;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<'a, G: DGainCalculatorGenerator<'a> + 'static, T: Gain<'a, G = G>> DGain<'a>
    for MaybeUninit<T>
{
    fn dyn_init(
        &mut self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &TransducerFilter,
    ) -> Result<Box<dyn DGainCalculatorGenerator<'a>>, GainError> {
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
pub struct BoxedGain<'geo> {
    g: Box<dyn DGain<'geo>>,
}

impl<'a> BoxedGain<'a> {
    /// Creates a new [`BoxedGain`].
    #[must_use]
    pub fn new<
        C: GainCalculator<'a> + 'static,
        GG: GainCalculatorGenerator<'a, Calculator = C> + 'static,
        G: Gain<'a, G = GG> + 'static,
    >(
        g: G,
    ) -> Self {
        Self {
            g: Box::new(MaybeUninit::new(g)),
        }
    }
}

impl std::fmt::Debug for BoxedGain<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.g.as_ref().dyn_fmt(f)
    }
}

impl<'a> Gain<'a> for BoxedGain<'a> {
    type G = DynGainCalculatorGenerator<'a>;

    fn init(
        self,
        geometry: &'a Geometry,
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

    use std::collections::HashMap;

    use crate::datagram::gain::tests::TestGain;

    use autd3_core::{
        firmware::Drive,
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
