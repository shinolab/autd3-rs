use autd3_core::derive::*;

use autd3_driver::{
    defined::rad,
    firmware::fpga::{EmitIntensity, Phase},
    geometry::Point3,
};

/// The option of [`Focus`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusOption {
    /// The intensity of the beam.
    pub intensity: EmitIntensity,
    /// The phase offset of the beam.
    pub phase_offset: Phase,
}

impl Default for FocusOption {
    fn default() -> Self {
        Self {
            intensity: EmitIntensity::MAX,
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

pub struct Context {
    pub(crate) pos: Point3,
    pub(crate) intensity: EmitIntensity,
    pub(crate) phase_offset: Phase,
    pub(crate) wavenumber: f32,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        Drive {
            phase: Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad)
                + self.phase_offset,
            intensity: self.intensity,
        }
    }
}

impl GainContextGenerator for Focus {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            pos: self.pos,
            intensity: self.option.intensity,
            phase_offset: self.option.phase_offset,
            wavenumber: device.wavenumber(),
        }
    }
}

impl Gain for Focus {
    type G = Focus;

    fn init(self) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{create_geometry, random_point3};

    use super::*;
    use rand::Rng;
    fn focus_check(
        g: Focus,
        pos: Point3,
        intensity: EmitIntensity,
        phase_offset: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        let mut b = g.init()?;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let expected_phase =
                    Phase::from(-(tr.position() - pos).norm() * dev.wavenumber() * rad)
                        + phase_offset;
                let d = d.calc(tr);
                assert_eq!(expected_phase, d.phase);
                assert_eq!(intensity, d.intensity);
            });
        });

        Ok(())
    }

    #[test]
    fn test_focus() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let pos = random_point3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let g = Focus {
            pos,
            option: Default::default(),
        };
        focus_check(g, pos, EmitIntensity::MAX, Phase::ZERO, &geometry)?;

        let pos = random_point3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let intensity = EmitIntensity(rng.gen());
        let phase_offset = Phase(rng.gen());
        let g = Focus {
            pos,
            option: FocusOption {
                intensity,
                phase_offset,
            },
        };
        focus_check(g, pos, intensity, phase_offset, &geometry)?;

        Ok(())
    }
}
