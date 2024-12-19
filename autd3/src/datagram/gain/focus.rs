use autd3_driver::{
    defined::rad,
    derive::*,
    firmware::fpga::{EmitIntensity, Phase},
    geometry::Point3,
};
use derive_new::new;

/// Single focus
#[derive(Gain, Clone, PartialEq, Debug, Builder, new)]
pub struct Focus {
    #[get(ref)]
    /// The position of the focus
    pos: Point3,
    #[new(value = "EmitIntensity::MAX")]
    #[get]
    #[set(into)]
    /// The intensity of the focus
    intensity: EmitIntensity,
    #[new(value = "Phase::ZERO")]
    #[get]
    #[set(into)]
    /// The phase offset of the focus
    phase_offset: Phase,
}

pub struct Context {
    pub(crate) pos: Point3,
    pub(crate) intensity: EmitIntensity,
    pub(crate) phase_offset: Phase,
    pub(crate) wavenumber: f32,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        (
            Phase::from(-(self.pos - tr.position()).norm() * self.wavenumber * rad)
                + self.phase_offset,
            self.intensity,
        )
            .into()
    }
}

impl GainContextGenerator for Focus {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            pos: self.pos,
            intensity: self.intensity,
            phase_offset: self.phase_offset,
            wavenumber: device.wavenumber(),
        }
    }
}

impl Gain for Focus {
    type G = Focus;

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
        assert_eq!(&pos, g.pos());
        assert_eq!(intensity, g.intensity());
        assert_eq!(phase_offset, g.phase_offset());

        let mut b = g.init(geometry, None)?;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let expected_phase =
                    Phase::from(-(tr.position() - pos).norm() * dev.wavenumber() * rad)
                        + phase_offset;
                let d = d.calc(tr);
                assert_eq!(expected_phase, d.phase());
                assert_eq!(intensity, d.intensity());
            });
        });

        Ok(())
    }

    #[test]
    fn test_focus() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let f = random_point3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let g = Focus::new(f);
        focus_check(g, f, EmitIntensity::MAX, Phase::ZERO, &geometry)?;

        let f = random_point3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let intensity = EmitIntensity::new(rng.gen());
        let phase_offset = Phase::new(rng.gen());
        let g = Focus::new(f)
            .with_intensity(intensity)
            .with_phase_offset(phase_offset);
        focus_check(g, f, intensity, phase_offset, &geometry)?;

        Ok(())
    }
}
