use crate::defined::Angle;

use super::{UnitQuaternion, Vector3};

#[non_exhaustive]
pub enum EulerAngle {
    ZYZ(Angle, Angle, Angle),
}

impl From<EulerAngle> for UnitQuaternion {
    fn from(angle: EulerAngle) -> Self {
        match angle {
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
            assert_approx_eq::assert_approx_eq!($a.w, $b.w, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.i, $b.i, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.j, $b.j, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.k, $b.k, 1e-3);
        };
    }

    #[rstest::rstest]
    #[test]
    #[case(0., 0. * deg)]
    #[case(PI / 2., 90. * deg)]
    #[case(0., 0. * rad)]
    #[case(PI / 2., PI / 2. * rad)]
    fn test_to_radians(#[case] expected: f32, #[case] angle: Angle) {
        assert_approx_eq::assert_approx_eq!(expected, angle.radian());
    }

    #[rstest::rstest]
    #[test]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(90. * deg, 0. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 90. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 0. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 90. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::ZYZ(90. * deg, 90. * deg, 0. * deg))]
    fn test_rotation(#[case] expected: UnitQuaternion, #[case] angle: EulerAngle) {
        let angle: UnitQuaternion = angle.into();
        assert_approx_eq_quat!(expected, angle);
    }
}
