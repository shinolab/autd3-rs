use autd3_driver::{
    defined::Angle,
    derive::*,
    geometry::{UnitQuaternion, Vector3},
};

#[derive(Gain, Clone, PartialEq, Debug, Builder)]
pub struct Bessel {
    #[get(ref)]
    pos: Vector3,
    #[get(ref)]
    dir: Vector3,
    #[get]
    theta: Angle,
    #[get]
    #[set(into)]
    intensity: EmitIntensity,
    #[get]
    #[set(into)]
    phase_offset: Phase,
}

impl Bessel {
    pub const fn new(pos: Vector3, dir: Vector3, theta: Angle) -> Self {
        Self {
            pos,
            dir,
            theta,
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::new(0),
        }
    }
}

impl Gain for Bessel {
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        let rot = {
            let dir = self.dir.normalize();
            let v = Vector3::new(dir.y, -dir.x, 0.);
            let theta_v = v.norm().asin();
            v.try_normalize(1.0e-6)
                .map_or_else(UnitQuaternion::identity, |v| {
                    UnitQuaternion::from_scaled_axis(v * -theta_v)
                })
        };
        let theta = self.theta.radian();
        let pos = self.pos;
        let phase_offset = self.phase_offset;
        let intensity = self.intensity;
        Ok(Self::transform(move |dev| {
            let wavenumber = dev.wavenumber();
            move |tr| {
                let r = rot * (tr.position() - pos);
                let dist = theta.sin() * r.xy().norm() - theta.cos() * r.z;
                (
                    Phase::from(-dist * wavenumber * rad) + phase_offset,
                    intensity,
                )
            }
        }))
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
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
        dir: Vector3,
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

        let b = g.calc(geometry)?;
        geometry.iter().for_each(|dev| {
            let d = b(dev);
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
                let d = d(tr);
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

        let g = Bessel::new(Vector3::zeros(), Vector3::z(), 0. * rad);
        assert_eq!(EmitIntensity::MAX, g.intensity());
        assert_eq!(Phase::new(0), g.phase_offset());

        let f = random_vector3(-500.0..500.0, -500.0..500.0, 50.0..500.0);
        let d = random_vector3(-1.0..1.0, -1.0..1.0, -1.0..1.0).normalize();
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
