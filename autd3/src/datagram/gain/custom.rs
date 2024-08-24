use autd3_driver::{
    derive::*,
    firmware::fpga::Drive,
    geometry::{Device, Transducer},
};

use derive_more::Debug;

#[derive(Gain, Debug)]
#[debug("Custom (Gain)")]
pub struct Custom<
    'a,
    D: Into<Drive>,
    FT: Fn(&Transducer) -> D + Send + Sync + 'static,
    F: Fn(&Device) -> FT + 'a,
> {
    f: F,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<
        'a,
        D: Into<Drive>,
        FT: Fn(&Transducer) -> D + Send + Sync + 'static,
        F: Fn(&Device) -> FT + 'a,
    > Custom<'a, D, FT, F>
{
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
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
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        Ok(Self::transform(&self.f))
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
        let test_drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));
        let transducer_test = Custom::new(move |dev| {
            let dev_idx = dev.idx();
            move |tr| {
                if dev_idx == 0 && tr.idx() == test_id {
                    test_drive
                } else {
                    Drive::null()
                }
            }
        });

        let d = transducer_test.calc(&geometry)?;
        geometry.iter().for_each(|dev| {
            let d = d(dev);
            dev.iter().enumerate().for_each(|(idx, tr)| {
                if dev.idx() == 0 && idx == test_id {
                    assert_eq!(test_drive, d(tr));
                } else {
                    assert_eq!(Drive::null(), d(tr));
                }
            });
        });

        Ok(())
    }
}
