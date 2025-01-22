use autd3_core::derive::*;
use autd3_driver::{
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};

use derive_more::Debug;
use derive_new::new;

/// [`Gain`] to use arbitrary phases and intensities
///
/// # Examples
///
/// ```
/// use autd3::prelude::*;
/// use autd3::gain::Custom;
///
/// Custom::new(|dev| |tr| Drive { phase: Phase::ZERO, intensity: EmitIntensity::MAX });
/// ```
#[derive(Gain, Debug, new)]
#[debug("Custom (Gain)")]
pub struct Custom<'a, D, FT, F>
where
    D: Into<Drive>,
    FT: Fn(&Transducer) -> D + Send + Sync + 'static,
    F: Fn(&Device) -> FT + 'a,
{
    pub f: F,
    _phantom: std::marker::PhantomData<&'a ()>,
}

pub struct Context<D: Into<Drive>, FT: Fn(&Transducer) -> D + Send + Sync + 'static> {
    f: FT,
}

impl<D: Into<Drive>, FT: Fn(&Transducer) -> D + Send + Sync + 'static> GainContext
    for Context<D, FT>
{
    fn calc(&self, tr: &Transducer) -> Drive {
        (self.f)(tr).into()
    }
}

impl<
        'a,
        D: Into<Drive>,
        FT: Fn(&Transducer) -> D + Send + Sync + 'static,
        F: Fn(&Device) -> FT + 'a,
    > GainContextGenerator for Custom<'a, D, FT, F>
{
    type Context = Context<D, FT>;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            f: (self.f)(device),
        }
    }
}

impl<
        'a,
        D: Into<Drive>,
        FT: Fn(&Transducer) -> D + Send + Sync + 'static,
        F: Fn(&Device) -> FT + 'a,
    > Gain for Custom<'a, D, FT, F>
{
    type G = Custom<'a, D, FT, F>;

    fn init(self) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;

    use crate::tests::create_geometry;

    use super::*;

    #[test]
    fn test_custom() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(2);

        let test_id = rng.gen_range(0..geometry[0].num_transducers());
        let test_drive = Drive {
            phase: Phase(rng.gen()),
            intensity: EmitIntensity(rng.gen()),
        };
        let transducer_test = Custom::new(move |dev| {
            let dev_idx = dev.idx();
            move |tr| {
                if dev_idx == 0 && tr.idx() == test_id {
                    test_drive
                } else {
                    Drive::NULL
                }
            }
        });

        let mut d = transducer_test.init()?;
        geometry.iter().for_each(|dev| {
            let d = d.generate(dev);
            dev.iter().enumerate().for_each(|(idx, tr)| {
                if dev.idx() == 0 && idx == test_id {
                    assert_eq!(test_drive, d.calc(tr));
                } else {
                    assert_eq!(Drive::NULL, d.calc(tr));
                }
            });
        });

        Ok(())
    }
}
