use autd3_core::derive::*;

use autd3_driver::{
    defined::rad,
    firmware::fpga::{EmitIntensity, Phase},
    geometry::UnitVector3,
};

/// The option of [`Plane`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct PlaneOption {
    /// The intensity of the beam.
    pub intensity: EmitIntensity,
    /// The phase offset of the beam.
    pub phase_offset: Phase,
}

impl Default for PlaneOption {
    fn default() -> Self {
        Self {
            intensity: EmitIntensity::MAX,
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

pub struct Context {
    dir: UnitVector3,
    intensity: EmitIntensity,
    phase_offset: Phase,
    wavenumber: f32,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        Drive {
            phase: Phase::from(-self.dir.dot(&tr.position().coords) * self.wavenumber * rad)
                + self.phase_offset,
            intensity: self.intensity,
        }
    }
}

impl GainContextGenerator for Plane {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            dir: self.dir,
            intensity: self.option.intensity,
            phase_offset: self.option.phase_offset,
            wavenumber: device.wavenumber(),
        }
    }
}

impl Gain for Plane {
    type G = Plane;

    fn init(self) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    use crate::tests::{create_geometry, random_vector3};

    fn plane_check(
        g: Plane,
        dir: UnitVector3,
        intensity: EmitIntensity,
        phase_offset: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        let mut b = g.init()?;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let expected_phase =
                    Phase::from(-dir.dot(&tr.position().coords) * dev.wavenumber() * rad)
                        + phase_offset;
                let d = d.calc(tr);
                assert_eq!(expected_phase, d.phase);
                assert_eq!(intensity, d.intensity);
            });
        });

        Ok(())
    }

    #[test]
    fn test_plane() -> anyhow::Result<()> {
        let mut rng = rand::rng();

        let geometry = create_geometry(1);

        let dir = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let g = Plane {
            dir,
            option: PlaneOption::default(),
        };
        plane_check(g, dir, EmitIntensity::MAX, Phase::ZERO, &geometry)?;

        let dir = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let intensity = EmitIntensity(rng.random());
        let phase_offset = Phase(rng.random());
        let g = Plane {
            dir,
            option: PlaneOption {
                intensity,
                phase_offset,
            },
        };
        plane_check(g, dir, intensity, phase_offset, &geometry)?;

        Ok(())
    }
}
