/*
 * File: mod.rs
 * Project: acoustics
 * Created Date: 04/10/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 19/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

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

    #[test]
    fn propagate() {
        let mut rng = rand::thread_rng();

        let tr = crate::geometry::Transducer::new(0, Vector3::zeros(), UnitQuaternion::identity());

        let atten = rng.gen_range(0.0..1e-6);
        let c = rng.gen_range(300e3..400e3);
        let target = Vector3::new(
            rng.gen_range(-100.0..100.0),
            rng.gen_range(-100.0..100.0),
            rng.gen_range(-100.0..100.0),
        );

        let expect = {
            let dist = target.norm();
            let r = T4010A1_AMPLITUDE
                * TestDirectivity::directivity_from_tr(&tr, &target)
                * (-dist * atten).exp()
                / (4. * PI * dist);
            let phase = -tr.wavenumber(c) * dist;
            Complex::new(r * phase.cos(), r * phase.sin())
        };
        assert_complex_approx_eq!(
            expect,
            super::propagate::<TestDirectivity>(&tr, atten, c, &target)
        );
    }
}
