use super::{Matrix4, UnitQuaternion, Vector3, Vector4};

#[derive(Clone, Debug, PartialEq)]
pub struct Transducer {
    idx: usize,
    pos: Vector3,
}

impl Transducer {
    pub(crate) const fn new(idx: usize, pos: Vector3) -> Self {
        assert!(idx < 256);
        Self { idx, pos }
    }

    pub fn affine(&mut self, t: Vector3, r: UnitQuaternion) {
        let new_pos = Matrix4::from(r).append_translation(&t)
            * Vector4::new(self.pos[0], self.pos[1], self.pos[2], 1.0);
        self.pos = Vector3::new(new_pos[0], new_pos[1], new_pos[2]);
    }

    pub const fn position(&self) -> &Vector3 {
        &self.pos
    }

    pub const fn idx(&self) -> usize {
        self.idx
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use assert_approx_eq::assert_approx_eq;

    use super::*;

    macro_rules! assert_vec3_approx_eq {
        ($a:expr, $b:expr) => {
            assert_approx_eq!($a.x, $b.x, 1e-3);
            assert_approx_eq!($a.y, $b.y, 1e-3);
            assert_approx_eq!($a.z, $b.z, 1e-3);
        };
    }

    #[rstest::fixture]
    fn tr() -> Transducer {
        Transducer::new(0, Vector3::zeros())
    }

    #[rstest::rstest]
    #[test]
    #[case(0)]
    #[case(1)]
    fn idx(#[case] i: usize) {
        assert_eq!(i, Transducer::new(i, Vector3::zeros()).idx());
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
