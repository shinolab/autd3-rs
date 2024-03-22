use super::{UnitQuaternion, Vector3};

pub struct Deg;
pub struct Rad;

#[derive(Clone, Copy)]
pub enum Angle {
    Deg(f64),
    Rad(f64),
}

impl Angle {
    fn to_radians(self) -> f64 {
        match self {
            Self::Deg(a) => a.to_radians(),
            Self::Rad(a) => a,
        }
    }
}

impl std::ops::Mul<Deg> for f64 {
    type Output = Angle;

    fn mul(self, _rhs: Deg) -> Self::Output {
        Self::Output::Deg(self)
    }
}

impl std::ops::Mul<Rad> for f64 {
    type Output = Angle;

    fn mul(self, _rhs: Rad) -> Self::Output {
        Self::Output::Rad(self)
    }
}

pub enum EulerAngle {
    ZYZ(Angle, Angle, Angle),
}

impl From<EulerAngle> for UnitQuaternion {
    fn from(angle: EulerAngle) -> Self {
        match angle {
            EulerAngle::ZYZ(z1, y, z2) => {
                UnitQuaternion::from_axis_angle(&Vector3::z_axis(), z1.to_radians())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), y.to_radians())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), z2.to_radians())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defined::PI;

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
    #[case(0., 0. * Deg)]
    #[case(PI / 2., 90. * Deg)]
    #[case(0., 0. * Rad)]
    #[case(PI / 2., PI / 2. * Rad)]
    fn test_to_radians(#[case] expected: f64, #[case] angle: Angle) {
        assert_approx_eq::assert_approx_eq!(expected, angle.to_radians());
    }

    #[rstest::rstest]
    #[test]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(90. * Deg, 0. * Deg, 0. * Deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::ZYZ(0. * Deg, 90. * Deg, 0. * Deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(0. * Deg, 0. * Deg, 90. * Deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.), EulerAngle::ZYZ(0. * Deg, 90. * Deg, 90. * Deg))]
    #[case(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.) * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 2.), EulerAngle::ZYZ(90. * Deg, 90. * Deg, 0. * Deg))]
    fn test_rotation(#[case] expected: UnitQuaternion, #[case] angle: EulerAngle) {
        let angle: UnitQuaternion = angle.into();
        assert_approx_eq_quat!(expected, angle);
    }
}
