use std::collections::HashMap;

use autd3_driver::{common::EmitIntensity, derive::*, geometry::Geometry};

/// Gain with uniform emission intensity and phase
#[derive(Gain, Clone, PartialEq, Debug)]
pub struct Uniform {
    intensity: EmitIntensity,
    phase: Phase,
}

impl Uniform {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `intensity` - Emission intensity
    ///
    pub fn new(intensity: impl Into<EmitIntensity>) -> Self {
        Self {
            intensity: intensity.into(),
            phase: Phase::new(0),
        }
    }

    /// set phase
    ///
    /// # Arguments
    ///
    /// * `phase` - phase
    ///
    pub fn with_phase(self, phase: Phase) -> Self {
        Self { phase, ..self }
    }

    pub const fn intensity(&self) -> EmitIntensity {
        self.intensity
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }
}

impl Gain for Uniform {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, |_, _| Drive {
            phase: self.phase,
            intensity: self.intensity,
        }))
    }
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

        let b = g.calc(geometry, GainFilter::All)?;
        assert_eq!(geometry.num_devices(), b.len());
        b.iter().for_each(|(&idx, d)| {
            assert_eq!(geometry[idx].num_transducers(), d.len());
            d.iter().for_each(|d| {
                assert_eq!(phase, d.phase);
                assert_eq!(intensity, d.intensity);
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

    #[test]
    fn test_uniform_derive() {
        let gain = Uniform::new(0x1F);
        let gain2 = gain.clone();
        assert_eq!(gain, gain2);
        let _ = gain.operation();
    }
}
