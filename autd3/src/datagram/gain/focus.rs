use autd3_driver::{
    defined::rad,
    derive::*,
    firmware::fpga::{EmitIntensity, Phase},
    geometry::Vector3,
};

#[derive(Gain, Clone, PartialEq, Debug, Builder)]
pub struct Focus {
    #[get(ref)]
    pos: Vector3,
    #[get]
    #[set(into)]
    intensity: EmitIntensity,
    #[get]
    #[set(into)]
    phase_offset: Phase,
}

impl Focus {
    pub const fn new(pos: Vector3) -> Self {
        Self {
            pos,
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::ZERO,
        }
    }
}

impl Gain for Focus {
    fn calc(&self, _geometry: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
        let pos = self.pos;
        let intensity = self.intensity;
        let phase_offset = self.phase_offset;
        Ok(Self::transform(move |dev| {
            let wavenumber = dev.wavenumber();
            move |tr| {
                (
                    Phase::from(-(pos - tr.position()).norm() * wavenumber * rad) + phase_offset,
                    intensity,
                )
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{create_geometry, random_vector3};

    use super::*;
    use rand::Rng;

    fn focus_check(
        g: Focus,
        pos: Vector3,
        intensity: EmitIntensity,
        phase_offset: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        assert_eq!(&pos, g.pos());
        assert_eq!(intensity, g.intensity());
        assert_eq!(phase_offset, g.phase_offset());

        let mut b = g.calc(geometry)?;
        geometry.iter().for_each(|dev| {
            let d = b(dev);
            dev.iter().for_each(|tr| {
                let expected_phase =
                    Phase::from(-(tr.position() - pos).norm() * dev.wavenumber() * rad)
                        + phase_offset;
                let d = d(tr);
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

        let f = random_vector3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let g = Focus::new(f);
        focus_check(g, f, EmitIntensity::MAX, Phase::ZERO, &geometry)?;

        let f = random_vector3(-100.0..100.0, -100.0..100.0, 100.0..200.0);
        let intensity = EmitIntensity::new(rng.gen());
        let phase_offset = Phase::new(rng.gen());
        let g = Focus::new(f)
            .with_intensity(intensity)
            .with_phase_offset(phase_offset);
        focus_check(g, f, intensity, phase_offset, &geometry)?;

        Ok(())
    }
}
