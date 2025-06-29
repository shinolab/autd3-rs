use autd3_core::derive::*;

use autd3_driver::{common::rad, geometry::Point3};

/// The option of [`Focus`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct FocusOption {
    /// The intensity of the focus.
    pub intensity: Intensity,
    /// The phase offset of the focus.
    pub phase_offset: Phase,
}

impl Default for FocusOption {
    fn default() -> Self {
        Self {
            intensity: Intensity::MAX,
            phase_offset: Phase::ZERO,
        }
    }
}

/// Single focus
#[derive(Gain, Clone, PartialEq, Debug)]
pub struct Focus {
    /// The position of the focus
    pub pos: Point3,
    /// The option of the gain.
    pub option: FocusOption,
}

impl Focus {
    /// Create a new [`Focus`].
    #[must_use]
    pub const fn new(pos: Point3, option: FocusOption) -> Self {
        Self { pos, option }
    }
}

#[derive(Clone, Copy)]
pub struct Impl {
    pub(crate) pos: Point3,
    pub(crate) intensity: Intensity,
    pub(crate) phase_offset: Phase,
    pub(crate) wavenumber: f32,
}

impl GainCalculator for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        Drive {
            phase: Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad)
                + self.phase_offset,
            intensity: self.intensity,
        }
    }
}

impl GainCalculatorGenerator for Impl {
    type Calculator = Impl;

    fn generate(&mut self, _: &Device) -> Self::Calculator {
        *self
    }
}

impl Gain for Focus {
    type G = Impl;

    fn init(
        self,
        _: &Geometry,
        env: &Environment,
        _: &TransducerFilter,
    ) -> Result<Self::G, GainError> {
        Ok(Impl {
            pos: self.pos,
            intensity: self.option.intensity,
            phase_offset: self.option.phase_offset,
            wavenumber: env.wavenumber(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{create_geometry, random_point3};

    use super::*;
    use rand::Rng;
    fn focus_check(
        mut b: Impl,
        pos: Point3,
        intensity: Intensity,
        phase_offset: Phase,
        geometry: &Geometry,
        env: &Environment,
    ) {
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let expected_phase =
                    Phase::from(-(tr.position() - pos).norm() * env.wavenumber() * rad)
                        + phase_offset;
                let d = d.calc(tr);
                assert_eq!(expected_phase, d.phase);
                assert_eq!(intensity, d.intensity);
            });
        });
    }

    #[test]
    fn test_focus() {
        let mut rng = rand::rng();

        let geometry = create_geometry(1);
        let env = Environment::new();

        let pos = random_point3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let g = Focus::new(pos, Default::default());
        focus_check(
            g.init(&geometry, &env, &TransducerFilter::all_enabled())
                .unwrap(),
            pos,
            Intensity::MAX,
            Phase::ZERO,
            &geometry,
            &env,
        );

        let pos = random_point3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let intensity = Intensity(rng.random());
        let phase_offset = Phase(rng.random());
        let g = Focus {
            pos,
            option: FocusOption {
                intensity,
                phase_offset,
            },
        };
        focus_check(
            g.init(&geometry, &env, &TransducerFilter::all_enabled())
                .unwrap(),
            pos,
            intensity,
            phase_offset,
            &geometry,
            &env,
        );
    }
}
