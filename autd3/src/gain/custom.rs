use std::collections::HashMap;

use autd3_driver::derive::*;

#[derive(Gain)]
pub struct Custom<FT: Fn(&Transducer) -> Drive + 'static, F: Fn(&Device) -> FT + Sync + 'static> {
    f: F,
}

impl<FT: Fn(&Transducer) -> Drive + 'static, F: Fn(&Device) -> FT + Sync + 'static> Custom<FT, F> {
    /// constructor
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<FT: Fn(&Transducer) -> Drive + 'static, F: Fn(&Device) -> FT + Sync + 'static> Gain
    for Custom<FT, F>
{
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, &self.f))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::tests::create_geometry;
    use autd3_driver::datagram::Datagram;

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

        let drives = transducer_test.calc(&geometry, GainFilter::All)?;
        drives[&0].iter().enumerate().for_each(|(idx, &drive)| {
            if idx == test_id {
                assert_eq!(test_drive, drive);
            } else {
                assert_eq!(Drive::null(), drive);
            }
        });
        drives[&1].iter().for_each(|&drive| {
            assert_eq!(Drive::null(), drive);
        });

        Ok(())
    }

    // GRCOV_EXCL_START
    fn f(_: &Device) -> impl Fn(&Transducer) -> Drive {
        |_| Drive::null()
    }
    // GRCOV_EXCL_STOP

    #[test]
    fn test_transtest_derive() {
        let gain = Custom::new(f);
        let _ = gain.operation();
    }
}
