use autd3_driver::{derive::*, firmware::fpga::Drive};

#[derive(Gain, Clone, PartialEq, Debug, Builder)]
pub struct Uniform {
    #[get]
    drive: Drive,
}

impl Uniform {
    pub fn new(drive: impl Into<Drive>) -> Self {
        Self {
            drive: drive.into(),
        }
    }
}

impl Gain for Uniform {
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        let d = self.drive;
        Ok(Self::transform(move |_| move |_| d))
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;
    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;

    #[test]
    fn test_uniform() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let intensity = EmitIntensity::new(rng.gen());
        let phase = Phase::new(rng.gen());
        let g = Uniform::new((intensity, phase));

        assert_eq!(intensity, g.drive().intensity());
        assert_eq!(phase, g.drive().phase());

        let b = g.calc(&geometry)?;
        geometry.iter().for_each(|dev| {
            let d = b(dev);
            dev.iter().for_each(|tr| {
                let d = d(tr);
                assert_eq!(phase, d.phase());
                assert_eq!(intensity, d.intensity());
            });
        });
        Ok(())
    }
}
