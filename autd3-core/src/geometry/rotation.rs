use crate::common::Angle;

use super::{UnitQuaternion, Vector3};

#[derive(Debug, Clone, Copy)]
/// Euler angle (intrinsic)
pub enum EulerAngle {
    /// x-y-z euler angle.
    XYZ(Angle, Angle, Angle),
    /// x-z-y euler angle.
    XZY(Angle, Angle, Angle),
    /// y-x-z euler angle.
    YXZ(Angle, Angle, Angle),
    /// y-z-x euler angle.
    YZX(Angle, Angle, Angle),
    /// z-x-y euler angle.
    ZXY(Angle, Angle, Angle),
    /// z-y-x euler angle.
    ZYX(Angle, Angle, Angle),
    /// x-y-x euler angle.
    XYX(Angle, Angle, Angle),
    /// x-z-x euler angle.
    XZX(Angle, Angle, Angle),
    /// y-x-y euler angle.
    YXY(Angle, Angle, Angle),
    /// y-z-y euler angle.
    YZY(Angle, Angle, Angle),
    /// z-x-z euler angle.
    ZXZ(Angle, Angle, Angle),
    /// z-y-z euler angle.
    ZYZ(Angle, Angle, Angle),
}

impl EulerAngle {
    /// The rotation identity.
    #[must_use]
    pub const fn identity() -> Self {
        Self::XYZ(Angle::ZERO, Angle::ZERO, Angle::ZERO)
    }
}

impl From<EulerAngle> for UnitQuaternion {
    fn from(angle: EulerAngle) -> Self {
        match angle {
            EulerAngle::XYZ(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), third.radian())
            }
            EulerAngle::XZY(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), third.radian())
            }
            EulerAngle::YXZ(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::y_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), third.radian())
            }
            EulerAngle::YZX(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::y_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), third.radian())
            }
            EulerAngle::ZXY(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::z_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), third.radian())
            }
            EulerAngle::ZYX(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::z_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), third.radian())
            }
            EulerAngle::XYX(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), third.radian())
            }
            EulerAngle::XZX(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), third.radian())
            }
            EulerAngle::YXY(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::y_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), third.radian())
            }
            EulerAngle::YZY(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::y_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), third.radian())
            }
            EulerAngle::ZXZ(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::z_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), third.radian())
            }
            EulerAngle::ZYZ(first, second, third) => {
                UnitQuaternion::from_axis_angle(&Vector3::z_axis(), first.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), second.radian())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), third.radian())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{PI, deg, rad};

    macro_rules! assert_approx_eq_quat {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.w, $b.w, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.i, $b.i, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.j, $b.j, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.k, $b.k, epsilon = 1e-3);
        };
    }

    #[rstest::rstest]
    #[case(0., 0. * deg)]
    #[case(PI / 2., 90. * deg)]
    #[case(0., 0. * rad)]
    #[case(PI / 2., PI / 2. * rad)]
    fn to_radians(#[case] expected: f32, #[case] angle: Angle) {
        approx::assert_abs_diff_eq!(expected, angle.radian());
    }

    #[rstest::rstest]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.), EulerAngle::XYZ(90. * deg, 0. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::XYZ(0. * deg, 90. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::XYZ(0. * deg, 0. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::XYZ(0. * deg, 90. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::XYZ(90. * deg, 90. * deg, 0. * deg))]
    fn xyz_intrinsic(#[case] expected: UnitQuaternion, #[case] angle: EulerAngle) {
        let angle: UnitQuaternion = angle.into();
        assert_approx_eq_quat!(expected, angle);
    }

    #[rstest::rstest]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(90. * deg, 0. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 90. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 0. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(0. * deg, 90. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::ZYZ(90. * deg, 90. * deg, 0. * deg))]
    fn zyz_intrinsic(#[case] expected: UnitQuaternion, #[case] angle: EulerAngle) {
        let angle: UnitQuaternion = angle.into();
        assert_approx_eq_quat!(expected, angle);
    }

    #[test]
    fn identity() {
        let angle: UnitQuaternion = EulerAngle::identity().into();
        assert_eq!(UnitQuaternion::identity(), angle);
    }
}
