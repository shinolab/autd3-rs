use autd3_core::derive::*;

use autd3_driver::{common::rad, geometry::UnitVector3};

/// The option of [`Plane`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct PlaneOption {
    /// The intensity of the wave.
    pub intensity: Intensity,
    /// The phase offset of the wave.
    pub phase_offset: Phase,
}

impl Default for PlaneOption {
    fn default() -> Self {
        Self {
            intensity: Intensity::MAX,
            phase_offset: Phase::ZERO,
        }
    }
}

/// Plane wave
#[derive(Gain, Clone, PartialEq, Debug)]
pub struct Plane {
    /// The direction of the plane wave.
    pub dir: UnitVector3,
    /// The option of the gain.
    pub option: PlaneOption,
}

impl Plane {
    /// Create a new [`Plane`].
    #[must_use]
    pub const fn new(dir: UnitVector3, option: PlaneOption) -> Self {
        Self { dir, option }
    }
}

#[derive(Clone, Copy)]
pub struct Impl {
    dir: UnitVector3,
    intensity: Intensity,
    phase_offset: Phase,
    wavenumber: f32,
}

impl GainCalculator<'_> for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        Drive {
            phase: Phase::from(-self.dir.dot(&tr.position().coords) * self.wavenumber * rad)
                + self.phase_offset,
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

impl Gain<'_> for Plane {
    type G = Impl;

    fn init(
        self,
        _: &Geometry,
        env: &Environment,
        _: &TransducerFilter,
    ) -> Result<Self::G, GainError> {
        Ok(Impl {
            dir: self.dir,
            intensity: self.option.intensity,
            phase_offset: self.option.phase_offset,
            wavenumber: env.wavenumber(),
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    use crate::tests::{create_geometry, random_vector3};

    fn plane_check(
        mut b: Impl,
        dir: UnitVector3,
        intensity: Intensity,
        phase_offset: Phase,
        geometry: &Geometry,
        env: &Environment,
    ) {
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let expected_phase =
                    Phase::from(-dir.dot(&tr.position().coords) * env.wavenumber() * rad)
                        + phase_offset;
                let d = d.calc(tr);
                assert_eq!(expected_phase, d.phase);
                assert_eq!(intensity, d.intensity);
            });
        });
    }

    #[test]
    fn test_plane() {
        let mut rng = rand::rng();

        let geometry = create_geometry(1);
        let env = Environment::new();

        let dir = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let g = Plane::new(dir, PlaneOption::default());
        plane_check(
            g.init(&geometry, &env, &TransducerFilter::all_enabled())
                .unwrap(),
            dir,
            Intensity::MAX,
            Phase::ZERO,
            &geometry,
            &env,
        );

        let dir = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let intensity = Intensity(rng.random());
        let phase_offset = Phase(rng.random());
        let g = Plane {
            dir,
            option: PlaneOption {
                intensity,
                phase_offset,
            },
        };
        plane_check(
            g.init(&geometry, &env, &TransducerFilter::all_enabled())
                .unwrap(),
            dir,
            intensity,
            phase_offset,
            &geometry,
            &env,
        );
    }
}
