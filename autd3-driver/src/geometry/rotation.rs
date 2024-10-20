use crate::defined::Angle;

use super::{UnitQuaternion, Vector3};

#[non_exhaustive]
pub enum EulerAngle {
    XYZ(Angle, Angle, Angle),
    ZYZ(Angle, Angle, Angle),
}

impl From<EulerAngle> for UnitQuaternion {
    fn from(angle: EulerAngle) -> Self {
        match angle {
            EulerAngle::XYZ(x, y, z) => {
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), x.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), y.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), z.radian())
            }
            EulerAngle::ZYZ(z1, y, z2) => {
                UnitQuaternion::from_axis_angle(&Vector3::z_axis(), z1.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), y.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), z2.radian())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defined::{deg, rad, PI};

    macro_rules! assert_approx_eq_quat {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.w, $b.w, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.i, $b.i, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.j, $b.j, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.k, $b.k, epsilon = 1e-3);
        };
    }

    #[rstest::rstest]
    #[test]
    #[case(0., 0. * deg)]
    #[case(PI / 2., 90. * deg)]
    #[case(0., 0. * rad)]
    #[case(PI / 2., PI / 2. * rad)]
    fn test_to_radians(#[case] expected: f32, #[case] angle: Angle) {
        approx::assert_abs_diff_eq!(expected, angle.radian());
    }

    #[rstest::rstest]
    #[test]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.), EulerAngle::XYZ(90. * deg, 0. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::XYZ(0. * deg, 90. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::XYZ(0. * deg, 0. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::XYZ(0. * deg, 90. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::XYZ(90. * deg, 90. * deg, 0. * deg))]
    fn test_rotation_xyz(#[case] expected: UnitQuaternion, #[case] angle: EulerAngle) {
        let angle: UnitQuaternion = angle.into();
        assert_approx_eq_quat!(expected, angle);
    }

    #[rstest::rstest]
    #[test]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(90. * deg, 0. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 90. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 0. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 90. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::ZYZ(90. * deg, 90. * deg, 0. * deg))]
    fn test_rotation_zyz(#[case] expected: UnitQuaternion, #[case] angle: EulerAngle) {
        let angle: UnitQuaternion = angle.into();
        assert_approx_eq_quat!(expected, angle);
    }
}
