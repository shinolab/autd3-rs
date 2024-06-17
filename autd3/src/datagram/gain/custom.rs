use autd3_driver::derive::*;

#[derive(Gain)]
pub struct Custom<
    FT: Fn(&Transducer) -> Drive + Send + Sync + 'static,
    F: Fn(&Device) -> FT + 'static,
> {
    f: F,
}

impl<FT: Fn(&Transducer) -> Drive + Send + Sync + 'static, F: Fn(&Device) -> FT + 'static>
    Custom<FT, F>
{
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<FT: Fn(&Transducer) -> Drive + Send + Sync + 'static, F: Fn(&Device) -> FT + 'static> Gain
    for Custom<FT, F>
{
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        let f = &self.f;
        Ok(Self::transform(f))
    }

    #[tracing::instrument(skip(self, _geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
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
