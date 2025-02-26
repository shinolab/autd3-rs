use crate::defined::Angle;

use super::{UnitQuaternion, Vector3};

use paste::paste;

macro_rules! make_euler_angle_intrinsic {
    ($({$first:ident, $second:ident, $third:ident}),*) => {
        paste! {
            /// Euler angle (intrinsic)
            pub enum EulerAngleIntrinsic {
                $(
                    #[doc = stringify!($first-$second-$third)]
                    #[doc = "euler angle."]
                    [<$first:upper $second:upper $third:upper>](Angle, Angle, Angle),
                )*
            }

            impl EulerAngleIntrinsic {
                /// The rotation identity.
                #[must_use]
                pub const fn identity() -> Self {
                    Self::XYZ(Angle::ZERO, Angle::ZERO, Angle::ZERO)
                }
            }

            impl From<EulerAngleIntrinsic> for UnitQuaternion {
                fn from(angle: EulerAngleIntrinsic) -> Self {
                    match angle {
                        $(
                            EulerAngleIntrinsic::[<$first:upper $second:upper $third:upper>](first, second, third) => {
                                UnitQuaternion::from_axis_angle(&Vector3::[<$first _axis>](), first.radian())
                                    * UnitQuaternion::from_axis_angle(&Vector3::[<$second _axis>](), second.radian())
                                    * UnitQuaternion::from_axis_angle(&Vector3::[<$third _axis>](), third.radian())
                            }
                        )*
                    }
                }
            }
        }
    }
}

macro_rules! make_euler_angle_extrinsic {
    ($({$first:ident, $second:ident, $third:ident}),*) => {
        paste! {
            /// Euler angle (extrinsic)
            pub enum EulerAngleExtrinsic {
                $(
                    #[doc = stringify!($first-$second-$third)]
                    #[doc = "euler angle."]
                    [<$first:upper $second:upper $third:upper>](Angle, Angle, Angle),
                )*
            }

            impl EulerAngleExtrinsic {
                /// The rotation identity.
                #[must_use]
                pub const fn identity() -> Self {
                    Self::XYZ(Angle::ZERO, Angle::ZERO, Angle::ZERO)
                }
            }

            impl From<EulerAngleExtrinsic> for UnitQuaternion {
                fn from(angle: EulerAngleExtrinsic) -> Self {
                    match angle {
                        $(
                            EulerAngleExtrinsic::[<$first:upper $second:upper $third:upper>](first, second, third) => {
                                UnitQuaternion::from_axis_angle(&Vector3::[<$third _axis>](), third.radian())
                                    * UnitQuaternion::from_axis_angle(&Vector3::[<$second _axis>](), second.radian())
                                    * UnitQuaternion::from_axis_angle(&Vector3::[<$first _axis>](), first.radian())
                            }
                        )*
                    }
                }
            }
        }
    }
}

make_euler_angle_intrinsic!({x, y, z}, {x, z, y}, {y, x, z}, {y, z, x}, {z, x, y}, {z, y, x}, {x, y, x}, {x, z, x}, {y, x, y}, {y, z, y}, {z, x, z}, {z, y, z});
make_euler_angle_extrinsic!({x, y, z}, {x, z, y}, {y, x, z}, {y, z, x}, {z, x, y}, {z, y, x}, {x, y, x}, {x, z, x}, {y, x, y}, {y, z, y}, {z, x, z}, {z, y, z});

/// Euler angle (intrinsic)
pub type EulerAngle = EulerAngleIntrinsic;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defined::{PI, deg, rad};

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

    #[rstest::rstest]
    #[test]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.), EulerAngleExtrinsic::XYZ(90. * deg, 0. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngleExtrinsic::XYZ(0. * deg, 90. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngleExtrinsic::XYZ(0. * deg, 0. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngleExtrinsic::XYZ(0. * deg, 90. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.), EulerAngleExtrinsic::XYZ(90. * deg, 90. * deg, 0. * deg))]
    fn test_rotation_xyz_extrinsic(
        #[case] expected: UnitQuaternion,
        #[case] angle: EulerAngleExtrinsic,
    ) {
        let angle: UnitQuaternion = angle.into();
        assert_approx_eq_quat!(expected, angle);
    }

    #[rstest::rstest]
    #[test]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngleExtrinsic::ZYZ(90. * deg, 0. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngleExtrinsic::ZYZ(0. * deg, 90. * deg, 0. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngleExtrinsic::ZYZ(0. * deg, 0. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngleExtrinsic::ZYZ(0. * deg, 90. * deg, 90. * deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngleExtrinsic::ZYZ(90. * deg, 90. * deg, 0. * deg))]
    fn test_rotation_zyz_extrinsic(
        #[case] expected: UnitQuaternion,
        #[case] angle: EulerAngleExtrinsic,
    ) {
        let angle: UnitQuaternion = angle.into();
        assert_approx_eq_quat!(expected, angle);
    }

    #[rstest::rstest]
    #[case(EulerAngleExtrinsic::identity())]
    #[case(EulerAngleIntrinsic::identity())]
    #[test]
    fn identity(#[case] angle: impl Into<UnitQuaternion>) {
        let angle: UnitQuaternion = angle.into();
        assert_eq!(UnitQuaternion::identity(), angle);
    }
}
