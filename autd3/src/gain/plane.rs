use std::collections::HashMap;

use autd3_driver::{derive::*, geometry::Vector3};

/// Gain to produce a plane wave
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
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `dir` - direction of the plane wave
    ///
    pub const fn new(dir: Vector3) -> Self {
        Self {
            dir,
            intensity: EmitIntensity::MAX,
            phase_offset: Phase::new(0),
        }
    }
}

impl Gain for Plane {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, |dev| {
            let wavenumber = dev.wavenumber();
            move |tr| {
                Drive::new(
                    Phase::from(self.dir.dot(tr.position()) * wavenumber * rad) + self.phase_offset,
                    self.intensity,
                )
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;
    use autd3_driver::datagram::Datagram;

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

        let d = g.calc(geometry, GainFilter::All)?;
        assert_eq!(geometry.num_devices(), d.len());
        d.iter().for_each(|(&idx, d)| {
            assert_eq!(geometry[idx].num_transducers(), d.len());
            d.iter().zip(geometry[idx].iter()).for_each(|(d, tr)| {
                let expected_phase =
                    Phase::from(dir.dot(tr.position()) * geometry[idx].wavenumber() * rad)
                        + phase_offset;
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

    #[test]
    fn test_plane_derive() {
        let gain = Plane::new(Vector3::zeros());
        let gain2 = gain.clone();
        assert_eq!(gain, gain2);
        let _ = gain.operation();
    }
}
