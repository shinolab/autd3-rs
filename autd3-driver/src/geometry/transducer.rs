use super::{Matrix4, Quaternion, UnitQuaternion, Vector3, Vector4};

#[derive(Clone, Debug, PartialEq)]
pub struct Transducer {
    idx: usize,
    pos: Vector3,
    rot: UnitQuaternion,
}

impl Transducer {
    /// Create transducer
    pub(crate) const fn new(idx: usize, pos: Vector3, rot: UnitQuaternion) -> Self {
        assert!(idx < 256);
        Self { idx, pos, rot }
    }

    /// Affine transformation
    pub fn affine(&mut self, t: Vector3, r: UnitQuaternion) {
        let new_pos = Matrix4::from(r).append_translation(&t)
            * Vector4::new(self.pos[0], self.pos[1], self.pos[2], 1.0);
        self.pos = Vector3::new(new_pos[0], new_pos[1], new_pos[2]);
        self.rot = r * self.rot;
    }

    /// Get the position of the transducer
    pub const fn position(&self) -> &Vector3 {
        &self.pos
    }

    /// Get the rotation of the transducer
    pub const fn rotation(&self) -> &UnitQuaternion {
        &self.rot
    }

    fn get_direction(dir: Vector3, rotation: &UnitQuaternion) -> Vector3 {
        let dir: UnitQuaternion = UnitQuaternion::from_quaternion(Quaternion::from_imag(dir));
        (rotation * dir * rotation.conjugate()).imag().normalize()
    }

    /// Get the local index of the transducer
    pub fn x_direction(&self) -> Vector3 {
        Self::get_direction(Vector3::x(), self.rotation())
    }
    /// Get the y-direction of the transducer
    pub fn y_direction(&self) -> Vector3 {
        Self::get_direction(Vector3::y(), self.rotation())
    }
    /// Get the z-direction of the transducer
    pub fn z_direction(&self) -> Vector3 {
        Self::get_direction(Vector3::z(), self.rotation())
    }

    /// Get the axial direction of the transducer
    #[cfg(feature = "left_handed")]
    pub fn axial_direction(&self) -> Vector3 {
        -self.z_direction()
    }

    /// Get the axial direction of the transducer
    #[cfg(not(feature = "left_handed"))]
    pub fn axial_direction(&self) -> Vector3 {
        self.z_direction()
    }

    /// Get the local transducer index
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
        Transducer::new(0, Vector3::zeros(), UnitQuaternion::identity())
    }

    #[rstest::rstest]
    #[test]
    #[case(0)]
    #[case(1)]
    fn idx(#[case] i: usize) {
        assert_eq!(
            i,
            Transducer::new(i, Vector3::zeros(), UnitQuaternion::identity()).idx()
        );
    }

    #[rstest::rstest]
    #[test]
    fn affine(mut tr: Transducer) {
        let t = Vector3::new(40., 50., 60.);
        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        tr.affine(t, rot);

        let expect_x = Vector3::new(0., 1., 0.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = Vector3::new(0., 0., 1.);
        assert_vec3_approx_eq!(expect_x, tr.x_direction());
        assert_vec3_approx_eq!(expect_y, tr.y_direction());
        assert_vec3_approx_eq!(expect_z, tr.z_direction());

        let expect_pos = Vector3::zeros() + t;
        assert_vec3_approx_eq!(expect_pos, tr.position());
    }
}
