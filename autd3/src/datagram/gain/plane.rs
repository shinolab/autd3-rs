use autd3_driver::{
    defined::rad,
    derive::*,
    firmware::fpga::{EmitIntensity, Phase},
    geometry::UnitVector3,
};
use derive_new::new;

#[derive(Gain, Clone, PartialEq, Debug, Builder, new)]
pub struct Plane {
    #[get(ref)]
    dir: UnitVector3,
    #[new(value = "EmitIntensity::MAX")]
    #[get]
    #[set(into)]
    intensity: EmitIntensity,
    #[new(value = "Phase::ZERO")]
    #[get]
    #[set(into)]
    phase_offset: Phase,
}

pub struct Context {
    dir: UnitVector3,
    intensity: EmitIntensity,
    phase_offset: Phase,
    wavenumber: f32,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        (
            Phase::from(-self.dir.dot(tr.position()) * self.wavenumber * rad) + self.phase_offset,
            self.intensity,
        )
            .into()
    }
}

impl GainContextGenerator for Plane {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            dir: self.dir,
            intensity: self.intensity,
            phase_offset: self.phase_offset,
            wavenumber: device.wavenumber(),
        }
    }
}

impl Gain for Plane {
    type G = Plane;

    fn init(
        self,
        _geometry: &Geometry,
        _filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
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
        assert_eq!(&dir, g.dir());
        assert_eq!(intensity, g.intensity());
        assert_eq!(phase_offset, g.phase_offset());

        let mut b = g.init(geometry, None)?;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let expected_phase =
                    Phase::from(-dir.dot(tr.position()) * dev.wavenumber() * rad) + phase_offset;
                let d = d.calc(tr);
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

        let d = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let g = Plane::new(d);
        plane_check(g, d, EmitIntensity::MAX, Phase::ZERO, &geometry)?;

        let d = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let intensity = EmitIntensity::new(rng.gen());
        let phase_offset = Phase::new(rng.gen());
        let g = Plane::new(d)
            .with_intensity(intensity)
            .with_phase_offset(phase_offset);
        plane_check(g, d, intensity, phase_offset, &geometry)?;

        Ok(())
    }
}
