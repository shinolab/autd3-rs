use std::collections::HashMap;

use autd3_driver::{
    common::EmitIntensity,
    derive::*,
    geometry::{Geometry, UnitQuaternion, Vector3},
};

/// Gain to produce a Bessel beam
#[derive(Gain, Clone, Copy)]
pub struct Bessel {
    intensity: EmitIntensity,
    pos: Vector3,
    dir: Vector3,
    theta: float,
    phase: Phase,
}

impl Bessel {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `pos` - Start point of the beam (the apex of the conical wavefront of the beam)
    /// * `dir` - Direction of the beam
    /// * `theta` - Angle between the conical wavefront of the beam and the plane normal to `dir`
    ///
    pub const fn new(pos: Vector3, dir: Vector3, theta: float) -> Self {
        Self {
            pos,
            dir,
            theta,
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

    pub const fn dir(&self) -> Vector3 {
        self.dir
    }

    pub const fn theta(&self) -> float {
        self.theta
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }
}

impl Gain for Bessel {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let rot = {
            let dir = self.dir.normalize();
            let v = Vector3::new(dir.y, -dir.x, 0.);
            let theta_v = v.norm().asin();
            v.try_normalize(1.0e-6)
                .map_or_else(UnitQuaternion::identity, |v| {
                    UnitQuaternion::from_scaled_axis(v * -theta_v)
                })
        };
        Ok(Self::transform(geometry, filter, |dev, tr| {
            let r = rot * (tr.position() - self.pos);
            let dist = self.theta.sin() * (r.x * r.x + r.y * r.y).sqrt() - self.theta.cos() * r.z;
            let phase = dist * tr.wavenumber(dev.sound_speed) * Rad + self.phase;
            Drive {
                phase,
                intensity: self.intensity,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use autd3_driver::{autd3_device::AUTD3, defined::PI, geometry::IntoDevice};

    use super::*;

    use crate::tests::random_vector3;

    #[test]
    fn test_bessel() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let f = random_vector3(-500.0..500.0, -500.0..500.0, 50.0..500.0);
        let d = random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0).normalize();
        let mut rng = rand::thread_rng();
        let theta = rng.gen_range(-PI..PI);
        let g = Bessel::new(f, d, theta);
        assert_eq!(g.pos(), f);
        assert_eq!(g.dir(), d);
        assert_eq!(g.theta(), theta);
        assert_eq!(g.intensity(), EmitIntensity::MAX);
        let b = g.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(b[&0].len(), geometry.num_transducers());
        b[&0]
            .iter()
            .for_each(|d| assert_eq!(d.intensity.value(), 0xFF));
        b[&0].iter().zip(geometry[0].iter()).for_each(|(b, tr)| {
            let expected_phase = {
                let dir = d.normalize();
                let v = Vector3::new(dir.y, -dir.x, 0.);
                let theta_v = v.norm().asin();
                let rot = if let Some(v) = v.try_normalize(1.0e-6) {
                    UnitQuaternion::from_scaled_axis(v * -theta_v)
                } else {
                    UnitQuaternion::identity()
                };
                let r = tr.position() - f;
                let r = rot * r;
                let dist = theta.sin() * (r.x * r.x + r.y * r.y).sqrt() - theta.cos() * r.z;
                dist * tr.wavenumber(geometry[0].sound_speed) * Rad
            };
            assert_eq!(b.phase, expected_phase);
        });

        let f = random_vector3(-500.0..500.0, -500.0..500.0, 50.0..500.0);
        let d = random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0).normalize();
        let theta = rng.gen_range(-PI..PI);
        let g = Bessel::new(f, d, theta).with_intensity(0x1F);
        assert_eq!(g.pos(), f);
        assert_eq!(g.dir(), d);
        assert_eq!(g.theta(), theta);
        assert_eq!(g.intensity(), EmitIntensity::new(0x1F));
        let b = g.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(b[&0].len(), geometry.num_transducers());
        b[&0]
            .iter()
            .for_each(|b| assert_eq!(b.intensity.value(), 0x1F));
        b[&0].iter().zip(geometry[0].iter()).for_each(|(b, tr)| {
            let expected_phase = {
                let dir = d.normalize();
                let v = Vector3::new(dir.y, -dir.x, 0.);
                let theta_v = v.norm().asin();
                let rot = if let Some(v) = v.try_normalize(1.0e-6) {
                    UnitQuaternion::from_scaled_axis(v * -theta_v)
                } else {
                    UnitQuaternion::identity()
                };
                let r = tr.position() - f;
                let r = rot * r;
                let dist = theta.sin() * (r.x * r.x + r.y * r.y).sqrt() - theta.cos() * r.z;
                dist * tr.wavenumber(geometry[0].sound_speed) * Rad
            };
            assert_eq!(b.phase, expected_phase);
        });
    }

    #[test]
    fn test_bessel_derive() {
        let g = Bessel::new(Vector3::zeros(), Vector3::zeros(), 0.);
        let g2 = g.clone();
        assert_eq!(g.pos(), g2.pos());
        assert_eq!(g.dir(), g2.dir());
        assert_eq!(g.theta(), g2.theta());
        assert_eq!(g.intensity(), g2.intensity());
        let _ = g.operation();
    }
}
