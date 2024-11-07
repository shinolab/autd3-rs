use autd3_driver::{
    defined::{rad, Angle},
    derive::*,
    firmware::fpga::{EmitIntensity, Phase},
    geometry::{UnitQuaternion, UnitVector3, Vector3},
};
use derive_new::new;

#[derive(Gain, Clone, PartialEq, Debug, Builder, new)]
pub struct Bessel {
    #[get(ref)]
    pos: Vector3,
    #[get(ref)]
    dir: UnitVector3,
    #[get]
    theta: Angle,
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
    pos: Vector3,
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
        (
            Phase::from(-dist * self.wavenumber * rad) + self.phase_offset,
            self.intensity,
        )
            .into()
    }
}

impl GainContextGenerator for Bessel {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            pos: self.pos,
            intensity: self.intensity,
            phase_offset: self.phase_offset,
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

    fn init(
        self,
        _geometry: &Geometry,
        _filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use autd3_driver::defined::PI;

    use super::*;

    use crate::tests::{create_geometry, random_vector3};

    fn bessel_check(
        g: Bessel,
        pos: Vector3,
        dir: UnitVector3,
        theta: Angle,
        intensity: EmitIntensity,
        phase_offset: Phase,
        geometry: &Geometry,
    ) -> anyhow::Result<()> {
        assert_eq!(&pos, g.pos());
        assert_eq!(&dir, g.dir());
        assert_eq!(theta, g.theta());
        assert_eq!(intensity, g.intensity());
        assert_eq!(phase_offset, g.phase_offset());

        let mut b = g.init(geometry, None)?;
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
                assert_eq!(expected_phase, d.phase());
                assert_eq!(intensity, d.intensity());
            });
        });

        Ok(())
    }

    #[test]
    fn test_bessel() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let g = Bessel::new(Vector3::zeros(), Vector3::z_axis(), 0. * rad);
        assert_eq!(EmitIntensity::MAX, g.intensity());
        assert_eq!(Phase::ZERO, g.phase_offset());

        let f = random_vector3(-500.0..500.0, -500.0..500.0, 50.0..500.0);
        let d = UnitVector3::new_normalize(random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0));
        let theta = rng.gen_range(-PI..PI);
        let intensity = EmitIntensity::new(rng.gen());
        let phase_offset = Phase::new(rng.gen());
        let g = Bessel::new(f, d, theta * rad)
            .with_intensity(intensity)
            .with_phase_offset(phase_offset);
        bessel_check(g, f, d, theta * rad, intensity, phase_offset, &geometry)?;

        Ok(())
    }
}
