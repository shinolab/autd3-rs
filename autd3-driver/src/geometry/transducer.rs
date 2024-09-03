use autd3_derive::Builder;

use super::{Matrix4, UnitQuaternion, Vector3, Vector4};

#[derive(Clone, Debug, PartialEq, Builder)]
pub struct Transducer {
    local_idx: u8,
    global_idx: u32,
    #[get(ref)]
    position: Vector3,
}

impl Transducer {
    pub(crate) const fn new(local_idx: u8, global_idx: u32, position: Vector3) -> Self {
        Self {
            local_idx,
            global_idx,
            position,
        }
    }

    pub const fn local_idx(&self) -> usize {
        self.local_idx as _
    }

    pub const fn global_idx(&self) -> usize {
        self.global_idx as _
    }

    pub fn affine(&mut self, t: Vector3, r: UnitQuaternion) {
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
    #[cfg_attr(miri, ignore)]
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
