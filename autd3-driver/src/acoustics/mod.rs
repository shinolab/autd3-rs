pub mod directivity;

use crate::{
    defined::{Complex, PI, T4010A1_AMPLITUDE},
    geometry::{Transducer, Vector3},
};

use directivity::Directivity;

#[inline]
pub fn propagate<D: Directivity>(
    tr: &Transducer,
    wavenumber: f32,
    dir: &Vector3,
    target_pos: &Vector3,
) -> Complex {
    const P0: f32 = T4010A1_AMPLITUDE / (4. * PI);
    let diff = target_pos - tr.position();
    let dist = diff.norm();
    Complex::from_polar(
        P0 / dist * D::directivity_from_dir(dir, &diff),
        -wavenumber * dist,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    use crate::{defined::mm, derive::Device, geometry::UnitQuaternion};
    use directivity::tests::TestDirectivity;

    macro_rules! assert_complex_approx_eq {
        ($a:expr, $b:expr) => {
            assert_approx_eq::assert_approx_eq!($a.re, $b.re, 1e-3 / mm);
            assert_approx_eq::assert_approx_eq!($a.im, $b.im, 1e-3 / mm);
        };
    }

    #[rstest::fixture]
    fn tr() -> Transducer {
        let mut rng = rand::thread_rng();
        Transducer::new(
            0,
            Vector3::new(
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-100.0..100.0),
            ),
        )
    }

    #[rstest::fixture]
    fn rot() -> UnitQuaternion {
        let mut rng = rand::thread_rng();
        UnitQuaternion::from_axis_angle(
            &Vector3::x_axis(),
            rng.gen_range::<f32, _>(-180.0..180.0).to_radians(),
        ) * UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            rng.gen_range::<f32, _>(-180.0..180.0).to_radians(),
        ) * UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            rng.gen_range::<f32, _>(-180.0..180.0).to_radians(),
        )
    }

    #[rstest::fixture]
    fn target() -> Vector3 {
        let mut rng = rand::thread_rng();
        Vector3::new(
            rng.gen_range(-100.0..100.0),
            rng.gen_range(-100.0..100.0),
            rng.gen_range(-100.0..100.0),
        )
    }

    #[rstest::fixture]
    fn sound_speed() -> f32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(300e3..400e3)
    }

    #[rstest::rstest]
    #[test]
    fn test_propagate(tr: Transducer, rot: UnitQuaternion, target: Vector3, sound_speed: f32) {
        #[cfg(feature = "dynamic_freq")]
        crate::set_ultrasound_freq(crate::defined::FREQ_40K);
        let mut device = Device::new(0, rot, vec![tr.clone()]);
        device.sound_speed = sound_speed;
        let wavenumber = device.wavenumber();
        assert_complex_approx_eq!(
            {
                let diff = target - tr.position();
                let dist = diff.norm();
                let r = T4010A1_AMPLITUDE
                    * TestDirectivity::directivity_from_dir(device.axial_direction(), &diff)
                    / (4. * PI * dist);
                let phase = -wavenumber * dist;
                Complex::new(r * phase.cos(), r * phase.sin())
            },
            super::propagate::<TestDirectivity>(&tr, wavenumber, device.axial_direction(), &target)
        );
    }
}
