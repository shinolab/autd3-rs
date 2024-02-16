use std::collections::HashMap;

use autd3_driver::{
    common::EmitIntensity,
    derive::*,
    geometry::{Geometry, UnitQuaternion, Vector3},
};

/// Gain to produce a Bessel beam
#[derive(Gain, Clone, PartialEq, Debug)]
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
            Drive::new(
                dist * tr.wavenumber(dev.sound_speed) * Rad + self.phase,
                self.intensity,
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use autd3_driver::defined::PI;

    use super::*;

    use crate::tests::{create_geometry, random_vector3};

    fn bessel_check(
        g: Bessel,
        pos: Vector3,
        dir: Vector3,
        theta: float,
        intensity: EmitIntensity,
        phase: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        assert_eq!(pos, g.pos());
        assert_eq!(dir, g.dir());
        assert_eq!(theta, g.theta());
        assert_eq!(intensity, g.intensity());
        assert_eq!(phase, g.phase());

        let b = g.calc(geometry, GainFilter::All)?;
        assert_eq!(geometry.num_devices(), b.len());
        b.iter().for_each(|(&idx, d)| {
            assert_eq!(geometry[idx].num_transducers(), d.len());
            d.iter().zip(geometry[idx].iter()).for_each(|(d, tr)| {
                let expected_phase = {
                    let dir = dir.normalize();
                    let v = Vector3::new(dir.y, -dir.x, 0.);
                    let theta_v = v.norm().asin();
                    let rot = if let Some(v) = v.try_normalize(1.0e-6) {
                        UnitQuaternion::from_scaled_axis(v * -theta_v)
                    } else {
                        UnitQuaternion::identity()
                    };
                    let r = tr.position() - pos;
                    let r = rot * r;
                    let dist = theta.sin() * (r.x * r.x + r.y * r.y).sqrt() - theta.cos() * r.z;
                    dist * tr.wavenumber(geometry[0].sound_speed) * Rad + phase
                };
                assert_eq!(expected_phase, d.phase());
                assert_eq!(intensity, d.intensity());
            });
        });

        Ok(())
    }

    #[test]
    fn test_bessel() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let f = random_vector3(-500.0..500.0, -500.0..500.0, 50.0..500.0);
        let d = random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0).normalize();
        let theta = rng.gen_range(-PI..PI);
        let g = Bessel::new(f, d, theta);
        bessel_check(g, f, d, theta, EmitIntensity::MAX, Phase::new(0), &geometry)?;

        let f = random_vector3(-500.0..500.0, -500.0..500.0, 50.0..500.0);
        let d = random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0).normalize();
        let theta = rng.gen_range(-PI..PI);
        let intensity = EmitIntensity::new(rng.gen());
        let phase = Phase::new(rng.gen());
        let g = Bessel::new(f, d, theta)
            .with_intensity(intensity)
            .with_phase(phase);
        bessel_check(g, f, d, theta, intensity, phase, &geometry)?;

        Ok(())
    }

    #[test]
    fn test_bessel_derive() {
        let g = Bessel::new(Vector3::zeros(), Vector3::zeros(), 0.);
        let g2 = g.clone();
        assert_eq!(g.pos(), g2.pos());
        assert_eq!(g.dir(), g2.dir());
        assert_eq!(g.theta(), g2.theta());
        assert_eq!(g.intensity(), g2.intensity());
        assert_eq!(g.phase(), g2.phase());
        let _ = g.operation_with_segment(Segment::S0, true);
    }
}
