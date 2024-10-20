use autd3_derive::Builder;

use super::{Matrix4, UnitQuaternion, Vector3, Vector4};

use derive_new::new;

#[derive(Clone, Debug, PartialEq, Builder, new)]
#[new(visibility = "pub(crate)")]
pub struct Transducer {
    idx: u8,
    dev_idx: u16,
    #[get(ref)]
    position: Vector3,
}

impl Transducer {
    pub const fn idx(&self) -> usize {
        self.idx as _
    }

    pub const fn dev_idx(&self) -> usize {
        self.dev_idx as _
    }

    pub(super) fn affine(&mut self, t: Vector3, r: UnitQuaternion) {
        self.position = (Matrix4::from(r).append_translation(&t)
            * Vector4::new(self.position[0], self.position[1], self.position[2], 1.0))
        .xyz();
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    macro_rules! assert_vec3_approx_eq {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.x, $b.x, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.y, $b.y, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.z, $b.z, epsilon = 1e-3);
        };
    }

    #[rstest::fixture]
    fn tr() -> Transducer {
        Transducer::new(0, 0, Vector3::zeros())
    }

    #[rstest::rstest]
    #[test]
    fn affine(mut tr: Transducer) {
        let t = Vector3::new(40., 50., 60.);
        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        tr.affine(t, rot);

        let expect_pos = Vector3::zeros() + t;
        assert_vec3_approx_eq!(expect_pos, tr.position());
    }
}
