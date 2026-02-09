use autd3_core::derive::*;
use autd3_driver::{
    common::{Angle, rad},
    geometry::{Point3, UnitQuaternion, UnitVector3, Vector3},
};

/// The option of [`Bessel`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct BesselOption {
    /// The intensity of the beam.
    pub intensity: Intensity,
    /// The phase offset of the beam.
    pub phase_offset: Phase,
}

impl Default for BesselOption {
    fn default() -> Self {
        Self {
            intensity: Intensity::MAX,
            phase_offset: Phase::ZERO,
        }
    }
}

/// Bessel beam
///
/// This [`Gain`] generates a Bessel beam. See [Hasegawa, 2017](https://doi.org/10.1063/1.4985159) for more details.
#[derive(Gain, Clone, PartialEq, Debug)]
pub struct Bessel {
    /// The apex of the beam.
    pub apex: Point3,
    /// The direction of the beam.
    pub dir: UnitVector3,
    /// The angle between the plane perpendicular to the beam and the side of the virtual cone that generates the beam.
    pub theta: Angle,
    /// The option of the gain.
    pub option: BesselOption,
}

impl Bessel {
    /// Create a new [`Bessel`].
    #[must_use]
    pub const fn new(apex: Point3, dir: UnitVector3, theta: Angle, option: BesselOption) -> Self {
        Self {
            apex,
            dir,
            theta,
            option,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Impl {
    apex: Point3,
    intensity: Intensity,
    phase_offset: Phase,
    wavenumber: f32,
    rot: UnitQuaternion,
    theta: f32,
}

impl GainCalculator<'_> for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        let r = self.rot * (tr.position() - self.apex);
        let dist = self.theta.sin() * r.xy().norm() - self.theta.cos() * r.z;
        Drive {
            phase: Phase::from(-dist * self.wavenumber * rad) + self.phase_offset,
            intensity: self.intensity,
        }
    }
}

impl GainCalculatorGenerator<'_> for Impl {
    type Calculator = Impl;

    fn generate(&mut self, _: &Device) -> Self::Calculator {
        *self
    }
}

impl Gain<'_> for Bessel {
    type G = Impl;

    fn init(
        self,
        _: &Geometry,
        env: &Environment,
        _: &TransducerMask,
    ) -> Result<Self::G, GainError> {
        Ok(Impl {
            apex: self.apex,
            intensity: self.option.intensity,
            phase_offset: self.option.phase_offset,
            wavenumber: env.wavenumber(),
            rot: {
                let dir = self.dir.normalize();
                let v = Vector3::new(dir.y, -dir.x, 0.);
                let theta_v = v.norm().asin();
                v.try_normalize(1.0e-6)
                    .map_or_else(UnitQuaternion::identity, |v| {
                        UnitQuaternion::new(v * -theta_v)
                    })
            },
            theta: self.theta.radian(),
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::RngExt;

    use autd3_driver::common::PI;

    use super::*;

    use crate::tests::{create_geometry, random_point3, random_vector3};

    #[allow(clippy::too_many_arguments)]
    fn bessel_check(
        mut b: Impl,
        apex: Point3,
        dir: UnitVector3,
        theta: Angle,
        intensity: Intensity,
        phase_offset: Phase,
        geometry: &Geometry,
        env: &Environment,
    ) {
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
                            UnitQuaternion::new(v * -theta_v)
                        });
                    let r = tr.position() - apex;
                    let r = rot * r;
                    let dist = theta.radian().sin() * (r.x * r.x + r.y * r.y).sqrt()
                        - theta.radian().cos() * r.z;
                    Phase::from(-dist * env.wavenumber() * rad) + phase_offset
                };
                let d = d.calc(tr);
                assert_eq!(expected_phase, d.phase);
                assert_eq!(intensity, d.intensity);
            });
        });
    }

    #[test]
    fn bessel() -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::rng();

        let geometry = create_geometry(1);
        let env = Environment::new();

        let g = Bessel::new(
            Point3::origin(),
            Vector3::z_axis(),
            0. * rad,
            BesselOption::default(),
        );
        assert_eq!(Intensity::MAX, g.option.intensity);
        assert_eq!(Phase::ZERO, g.option.phase_offset);

        let apex = random_point3(-500.0..500.0, -500.0..500.0, 50.0..500.0);
        let dir = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let theta = rng.random_range(-PI..PI) * rad;
        let intensity = Intensity(rng.random());
        let phase_offset = Phase(rng.random());
        let g = Bessel {
            apex,
            dir,
            theta,
            option: BesselOption {
                intensity,
                phase_offset,
            },
        };
        bessel_check(
            g.init(&geometry, &env, &TransducerMask::AllEnabled)
                .unwrap(),
            apex,
            dir,
            theta,
            intensity,
            phase_offset,
            &geometry,
            &env,
        );

        Ok(())
    }
}
