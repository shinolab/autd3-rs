use autd3_driver::derive::*;

#[derive(Gain, Clone, PartialEq, Debug, Builder)]
pub struct Uniform {
    #[get]
    intensity: EmitIntensity,
    #[getset]
    phase: Phase,
}

impl Uniform {
    pub fn new(intensity: impl Into<EmitIntensity>) -> Self {
        Self {
            intensity: intensity.into(),
            phase: Phase::new(0),
        }
    }
}

impl Gain for Uniform {
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        let d = Drive::new(self.phase, self.intensity);
        Ok(Self::transform(move |_| move |_| d))
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::info!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;
    use rand::Rng;

    fn uniform_check(
        g: Uniform,
        intensity: EmitIntensity,
        phase: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        assert_eq!(intensity, g.intensity());
        assert_eq!(phase, g.phase());

        let b = g.calc(geometry)?;
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

    #[test]
    fn test_uniform() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let intensity = EmitIntensity::new(rng.gen());
        let g = Uniform::new(intensity);
        uniform_check(g, intensity, Phase::new(0), &geometry)?;

        let intensity = EmitIntensity::new(rng.gen());
        let phase = Phase::new(rng.gen());
        let g = Uniform::new(intensity).with_phase(phase);
        uniform_check(g, intensity, phase, &geometry)?;

        Ok(())
    }
}
