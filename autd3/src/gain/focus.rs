use std::collections::HashMap;

use autd3_driver::{derive::*, geometry::Vector3};

/// Gain to produce a focal point
#[derive(Gain, Clone, PartialEq, Debug)]
pub struct Focus {
    intensity: EmitIntensity,
    pos: Vector3,
    phase_offset: Phase,
}

impl Focus {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `pos` - position of the focal point
    ///
    pub const fn new(pos: Vector3) -> Self {
        Self {
            pos,
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::new(0),
        }
    }

    /// set emission intensity
    ///
    /// # Arguments
    ///
    /// * `intensity` - emission intensity
    ///
    pub fn with_intensity(self, intensity: impl Into<EmitIntensity>) -> Self {
        Self {
            intensity: intensity.into(),
            ..self
        }
    }

    /// set phase
    ///
    /// # Arguments
    ///
    /// * `phase_offset` - phase_offset
    ///
    pub fn with_phase_offset(self, phase_offset: Phase) -> Self {
        Self {
            phase_offset,
            ..self
        }
    }

    pub const fn intensity(&self) -> EmitIntensity {
        self.intensity
    }

    pub const fn pos(&self) -> Vector3 {
        self.pos
    }

    pub const fn phase_offset(&self) -> Phase {
        self.phase_offset
    }
}

impl Gain for Focus {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, |dev, tr| {
            Drive::new(
                tr.align_phase_at(self.pos, dev.sound_speed) + self.phase_offset,
                self.intensity,
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{create_geometry, random_vector3};

    use super::*;
    use rand::Rng;

    fn focus_check(
        g: Focus,
        pos: Vector3,
        intensity: EmitIntensity,
        phase_offset: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        assert_eq!(pos, g.pos());
        assert_eq!(intensity, g.intensity());
        assert_eq!(phase_offset, g.phase_offset());

        let b = g.calc(geometry, GainFilter::All)?;
        assert_eq!(geometry.num_devices(), b.len());
        b.iter().for_each(|(&idx, d)| {
            assert_eq!(d.len(), geometry[idx].num_transducers());
            d.iter().zip(geometry[idx].iter()).for_each(|(d, tr)| {
                let expected_phase = Phase::from_rad(
                    (tr.position() - pos).norm() * tr.wavenumber(geometry[idx].sound_speed),
                ) + phase_offset;
                assert_eq!(expected_phase, d.phase());
                assert_eq!(intensity, d.intensity())
            });
        });

        Ok(())
    }

    #[test]
    fn test_focus() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let f = random_vector3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let g = Focus::new(f);
        focus_check(g, f, EmitIntensity::MAX, Phase::new(0), &geometry)?;

        let f = random_vector3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let intensity = EmitIntensity::new(rng.gen());
        let phase_offset = Phase::new(rng.gen());
        let g = Focus::new(f)
            .with_intensity(intensity)
            .with_phase_offset(phase_offset);
        focus_check(g, f, intensity, phase_offset, &geometry)?;

        Ok(())
    }

    #[test]
    fn test_focus_derive() {
        let gain = Focus::new(Vector3::zeros());
        let gain2 = gain.clone();
        assert_eq!(gain, gain2);
        let _ = gain.operation_with_segment(Segment::S0, true);
    }
}
