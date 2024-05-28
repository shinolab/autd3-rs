use autd3_driver::{derive::*, geometry::Vector3};

#[derive(Gain, Clone, PartialEq, Debug, Builder)]
pub struct Plane {
    #[get]
    dir: Vector3,
    #[getset]
    intensity: EmitIntensity,
    #[getset]
    phase_offset: Phase,
}

impl Plane {
    pub const fn new(dir: Vector3) -> Self {
        Self {
            dir,
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::new(0),
        }
    }
}

impl Gain for Plane {
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        let dir = self.dir;
        let intensity = self.intensity;
        let phase_offset = self.phase_offset;
        Ok(Self::transform(move |dev| {
            let wavenumber = dev.wavenumber();
            move |tr| {
                Drive::new(
                    Phase::from(dir.dot(tr.position()) * wavenumber * rad) + phase_offset,
                    intensity,
                )
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    use crate::tests::{create_geometry, random_vector3};

    fn plane_check(
        g: Plane,
        dir: Vector3,
        intensity: EmitIntensity,
        phase_offset: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        assert_eq!(&dir, g.dir());
        assert_eq!(intensity, g.intensity());
        assert_eq!(phase_offset, g.phase_offset());

        let b = g.calc(geometry)?;
        geometry.iter().for_each(|dev| {
            let d = b(dev);
            dev.iter().for_each(|tr| {
                let expected_phase =
                    Phase::from(dir.dot(tr.position()) * dev.wavenumber() * rad) + phase_offset;
                let d = d(tr);
                assert_eq!(expected_phase, d.phase());
                assert_eq!(intensity, d.intensity());
            });
        });

        Ok(())
    }

    #[test]
    fn test_plane() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let d = random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0).normalize();
        let g = Plane::new(d);
        plane_check(g, d, EmitIntensity::MAX, Phase::new(0), &geometry)?;

        let d = random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0).normalize();
        let intensity = EmitIntensity::new(rng.gen());
        let phase_offset = Phase::new(rng.gen());
        let g = Plane::new(d)
            .with_intensity(intensity)
            .with_phase_offset(phase_offset);
        plane_check(g, d, intensity, phase_offset, &geometry)?;

        Ok(())
    }
}
