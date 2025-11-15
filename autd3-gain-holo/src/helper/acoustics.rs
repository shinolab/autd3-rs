use core::f32::consts::PI;

use autd3_core::{
    acoustics::directivity::Directivity,
    common::{T4010A1_AMPLITUDE, rad},
    geometry::{Complex, Point3, Transducer, UnitVector3, Vector3},
};

#[must_use]
fn directivity_from_dir<D: Directivity>(axial_direction: UnitVector3, target: Vector3) -> f32 {
    D::directivity(
        (axial_direction.cross(&target).norm()).atan2(axial_direction.dot(&target)) * rad,
    )
}

/// Calculate the pressure at the target position.
#[inline]
#[must_use]
pub fn propagate<D: Directivity>(
    tr: &Transducer,
    wavenumber: f32,
    dir: UnitVector3,
    target_pos: Point3,
) -> Complex {
    const P0: f32 = T4010A1_AMPLITUDE / (4. * PI);
    let diff = target_pos - tr.position();
    let dist = diff.norm();
    let r = P0 / dist * directivity_from_dir::<D>(dir, diff);
    let theta = wavenumber * dist;
    Complex::new(r * theta.cos(), r * theta.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    use autd3_core::{
        acoustics::directivity::Sphere,
        common::{ULTRASOUND_FREQ, mm},
        geometry::{Device, UnitQuaternion, Vector3},
    };

    macro_rules! assert_complex_approx_eq {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.re, $b.re, epsilon = 1e-3 / mm);
            approx::assert_abs_diff_eq!($a.im, $b.im, epsilon = 1e-3 / mm);
        };
    }

    #[rstest::fixture]
    fn tr() -> Transducer {
        let mut rng = rand::rng();
        Transducer::new(Point3::new(
            rng.random_range(-100.0..100.0),
            rng.random_range(-100.0..100.0),
            rng.random_range(-100.0..100.0),
        ))
    }

    #[rstest::fixture]
    fn rot() -> UnitQuaternion {
        let mut rng = rand::rng();
        UnitQuaternion::from_axis_angle(
            &Vector3::x_axis(),
            rng.random_range::<f32, _>(-180.0..180.0).to_radians(),
        ) * UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            rng.random_range::<f32, _>(-180.0..180.0).to_radians(),
        ) * UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            rng.random_range::<f32, _>(-180.0..180.0).to_radians(),
        )
    }

    #[rstest::fixture]
    fn target() -> Point3 {
        let mut rng = rand::rng();
        Point3::new(
            rng.random_range(-100.0..100.0),
            rng.random_range(-100.0..100.0),
            rng.random_range(-100.0..100.0),
        )
    }

    #[rstest::fixture]
    fn sound_speed() -> f32 {
        let mut rng = rand::rng();
        rng.random_range(300e3..400e3)
    }

    #[rstest::rstest]
    fn propagate(tr: Transducer, rot: UnitQuaternion, target: Point3, sound_speed: f32) {
        let device = Device::new(rot, vec![tr.clone()]);
        let wavelength = sound_speed / ULTRASOUND_FREQ.hz() as f32;
        let wavenumber = 2. * PI / wavelength;
        assert_complex_approx_eq!(
            {
                let diff = target - tr.position();
                let dist = diff.norm();
                let r = T4010A1_AMPLITUDE / (4. * PI) / dist
                    * directivity_from_dir::<Sphere>(device.axial_direction(), diff);
                let phase = wavenumber * dist;
                Complex::new(r * phase.cos(), r * phase.sin())
            },
            super::propagate::<Sphere>(&tr, wavenumber, device.axial_direction(), target)
        );
    }
}
