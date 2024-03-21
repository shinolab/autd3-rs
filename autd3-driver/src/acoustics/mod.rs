pub mod directivity;

use crate::{
    defined::{float, Complex, PI, T4010A1_AMPLITUDE},
    geometry::{Transducer, Vector3},
};

use directivity::Directivity;

/// Calculate propagation of ultrasound wave
///
/// # Arguments
///
/// * `tr` - Source [Transducer]
/// * `attenuation` - Attenuation coefficient
/// * `sound_speed` - Speed of sound
/// * `target_pos` - Position of target
///
pub fn propagate<D: Directivity>(
    tr: &Transducer,
    attenuation: float,
    sound_speed: float,
    target_pos: &Vector3,
) -> Complex {
    let diff = target_pos - tr.position();
    let dist = diff.norm();
    Complex::from_polar(
        T4010A1_AMPLITUDE / (4. * PI) / dist
            * D::directivity_from_tr(tr, &diff)
            * (-dist * attenuation).exp(),
        -tr.wavenumber(sound_speed) * dist,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    use crate::geometry::UnitQuaternion;
    use directivity::tests::TestDirectivity;

    macro_rules! assert_complex_approx_eq {
        ($a:expr, $b:expr) => {
            assert_approx_eq::assert_approx_eq!($a.re, $b.re);
            assert_approx_eq::assert_approx_eq!($a.im, $b.im);
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
            UnitQuaternion::from_axis_angle(
                &Vector3::x_axis(),
                rng.gen_range::<float, _>(-180.0..180.0).to_radians(),
            ) * UnitQuaternion::from_axis_angle(
                &Vector3::y_axis(),
                rng.gen_range::<float, _>(-180.0..180.0).to_radians(),
            ) * UnitQuaternion::from_axis_angle(
                &Vector3::z_axis(),
                rng.gen_range::<float, _>(-180.0..180.0).to_radians(),
            ),
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
    fn attenuation() -> float {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1e-6)
    }

    #[rstest::fixture]
    fn sound_speed() -> float {
        let mut rng = rand::thread_rng();
        rng.gen_range(300e3..400e3)
    }

    #[rstest::rstest]
    #[test]
    fn test_propagate(tr: Transducer, target: Vector3, attenuation: float, sound_speed: float) {
        assert_complex_approx_eq!(
            {
                let diff = target - tr.position();
                let dist = diff.norm();
                let r = T4010A1_AMPLITUDE
                    * TestDirectivity::directivity_from_tr(&tr, &diff)
                    * (-dist * attenuation).exp()
                    / (4. * PI * dist);
                let phase = -tr.wavenumber(sound_speed) * dist;
                Complex::new(r * phase.cos(), r * phase.sin())
            },
            super::propagate::<TestDirectivity>(&tr, attenuation, sound_speed, &target)
        );
    }
}
