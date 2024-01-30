use std::collections::HashMap;

use autd3_driver::{
    common::EmitIntensity,
    derive::*,
    geometry::{Geometry, Vector3},
};

/// Gain to produce a focal point
#[derive(Gain, Clone, Copy)]
pub struct Focus {
    intensity: EmitIntensity,
    pos: Vector3,
    phase: Phase,
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
            phase: Phase::new(0),
        }
    }

    /// set emission intensity
    ///
    /// # Arguments
    ///
    /// * `intensity` - emission intensity
    ///
    pub fn with_intensity<A: Into<EmitIntensity>>(self, intensity: A) -> Self {
        Self {
            intensity: intensity.into(),
            ..self
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

    pub const fn pos(&self) -> Vector3 {
        self.pos
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }
}

impl Gain for Focus {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, |dev, tr| Drive {
            phase: tr.align_phase_at(self.pos, dev.sound_speed) + self.phase,
            intensity: self.intensity,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::random_vector3;

    use super::*;
    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{IntoDevice, Vector3},
    };

    #[test]
    fn test_focus() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let f = random_vector3(-100.0..100.0, -100.0..100.0, 100.0..200.0);

        let g = Focus::new(f);
        assert_eq!(g.pos(), f);
        assert_eq!(g.intensity(), EmitIntensity::MAX);

        let d = g.calc(&geometry, GainFilter::All).unwrap();
        d[&0].iter().for_each(|drive| {
            assert_eq!(drive.intensity, EmitIntensity::MAX);
        });

        let g = g.with_intensity(0x1F);
        assert_eq!(g.intensity(), EmitIntensity::new(0x1F));
        let d = g.calc(&geometry, GainFilter::All).unwrap();
        d[&0].iter().for_each(|drive| {
            assert_eq!(drive.intensity, EmitIntensity::new(0x1F));
        });
    }

    #[test]
    fn test_focus_derive() {
        let gain = Focus::new(Vector3::zeros());
        let gain2 = gain.clone();
        assert_eq!(gain.pos(), gain2.pos());
        assert_eq!(gain.intensity(), gain2.intensity());
        let _ = gain.operation();
    }
}
