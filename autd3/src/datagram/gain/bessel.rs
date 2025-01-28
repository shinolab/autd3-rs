use autd3_core::derive::*;
use autd3_driver::{
    defined::{rad, Angle},
    firmware::fpga::{EmitIntensity, Phase},
    geometry::{Point3, UnitQuaternion, UnitVector3, Vector3},
};

/// The option of [`Bessel`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct BesselOption {
    /// The intensity of the beam.
    pub intensity: EmitIntensity,
    /// The phase offset of the beam.
    pub phase_offset: Phase,
}

impl Default for BesselOption {
    fn default() -> Self {
        Self {
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::ZERO,
        }
    }
}

/// Bessel beam
///
/// This [`Gain`] generates a Bessel beam. See [Hasegawa, 2017](https://doi.org/10.1063/1.4985159) for more details.
#[derive(Gain, Clone, PartialEq, Debug)]
pub struct Bessel {
    /// The vertex of the beam.
    pub pos: Point3,
    /// The direction of the beam.
    pub dir: UnitVector3,
    /// The angle between the plane perpendicular to the beam and the side of the virtual cone that generates the beam.
    pub theta: Angle,
    /// The option of the gain.
    pub option: BesselOption,
}

pub struct Context {
    pos: Point3,
    intensity: EmitIntensity,
    phase_offset: Phase,
    wavenumber: f32,
    rot: UnitQuaternion,
    theta: f32,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        let r = self.rot * (tr.position() - self.pos);
        let dist = self.theta.sin() * r.xy().norm() - self.theta.cos() * r.z;
        Drive {
            phase: Phase::from(-dist * self.wavenumber * rad) + self.phase_offset,
            intensity: self.intensity,
        }
    }
}

impl GainContextGenerator for Bessel {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            pos: self.pos,
            intensity: self.option.intensity,
            phase_offset: self.option.phase_offset,
            wavenumber: device.wavenumber(),
            rot: {
                let dir = self.dir.normalize();
                let v = Vector3::new(dir.y, -dir.x, 0.);
                let theta_v = v.norm().asin();
                v.try_normalize(1.0e-6)
                    .map_or_else(UnitQuaternion::identity, |v| {
                        UnitQuaternion::from_scaled_axis(v * -theta_v)
                    })
            },
            theta: self.theta.radian(),
        }
    }
}

impl Gain for Bessel {
    type G = Bessel;

    fn init(self) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use autd3_driver::defined::PI;

    use super::*;

    use crate::tests::{create_geometry, random_point3, random_vector3};

    fn bessel_check(
        g: Bessel,
        pos: Point3,
        dir: UnitVector3,
        theta: Angle,
        intensity: EmitIntensity,
        phase_offset: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        let mut b = g.init()?;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let expected_phase = {
                    let dir = dir.normalize();
                    let v = Vector3::new(dir.y, -dir.x, 0.);
                    let theta_v = v.norm().asin();
                    let rot = v
                        .try_normalize(1.0e-6)
                        .map_or_else(UnitQuaternion::identity, |v| {
                            UnitQuaternion::from_scaled_axis(v * -theta_v)
                        });
                    let r = tr.position() - pos;
                    let r = rot * r;
                    let dist = theta.radian().sin() * (r.x * r.x + r.y * r.y).sqrt()
                        - theta.radian().cos() * r.z;
                    Phase::from(-dist * geometry[0].wavenumber() * rad) + phase_offset
                };
                let d = d.calc(tr);
                assert_eq!(expected_phase, d.phase);
                assert_eq!(intensity, d.intensity);
            });
        });

        Ok(())
    }

    #[test]
    fn test_bessel() -> anyhow::Result<()> {
        let mut rng = rand::rng();

        let geometry = create_geometry(1);

        let g = Bessel {
            pos: Point3::origin(),
            dir: Vector3::z_axis(),
            theta: 0. * rad,
            option: BesselOption::default(),
        };
        assert_eq!(EmitIntensity::MAX, g.option.intensity);
        assert_eq!(Phase::ZERO, g.option.phase_offset);

        let pos = random_point3(-500.0..500.0, -500.0..500.0, 50.0..500.0);
        let dir = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let theta = rng.random_range(-PI..PI) * rad;
        let intensity = EmitIntensity(rng.random());
        let phase_offset = Phase(rng.random());
        let g = Bessel {
            pos,
            dir,
            theta,
            option: BesselOption {
                intensity,
                phase_offset,
            },
        };
        bessel_check(g, pos, dir, theta, intensity, phase_offset, &geometry)?;

        Ok(())
    }
}
