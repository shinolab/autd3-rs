use std::f32::consts::PI;

use crate::{
    defined::{Angle, rad},
    geometry::Complex,
};

use derive_more::Debug;
use nalgebra::ComplexField;
use zerocopy::{Immutable, IntoBytes};

/// The phase of the ultrasound.
#[derive(Clone, Copy, PartialEq, Eq, Debug, IntoBytes, Immutable, Default)]
#[repr(C)]
#[debug("{:#04X}", self.0)]
pub struct Phase(pub u8);

impl Phase {
    /// A phase of zero.
    pub const ZERO: Self = Self(0);
    /// A phase of π.
    pub const PI: Self = Self(0x80);

    /// Converts the phase into a radian.
    pub fn radian(&self) -> f32 {
        self.0 as f32 / 256.0 * 2.0 * PI
    }
}

impl From<Angle> for Phase {
    fn from(v: Angle) -> Self {
        Self((((v.radian() / (2.0 * PI) * 256.0).round() as i32) & 0xFF) as _)
    }
}

impl From<Complex> for Phase {
    fn from(v: Complex) -> Self {
        Self::from(v.argument() * rad)
    }
}

impl std::ops::Add<Phase> for Phase {
    type Output = Phase;

    fn add(self, rhs: Phase) -> Self::Output {
        Phase(self.0.wrapping_add(rhs.0))
    }
}

impl std::ops::Sub<Phase> for Phase {
    type Output = Phase;

    fn sub(self, rhs: Phase) -> Self::Output {
        Phase(self.0.wrapping_sub(rhs.0))
    }
}

impl std::ops::Mul<u8> for Phase {
    type Output = Phase;

    fn mul(self, rhs: u8) -> Self::Output {
        Phase(self.0.wrapping_mul(rhs))
    }
}

impl std::ops::Div<u8> for Phase {
    type Output = Phase;

    fn div(self, rhs: u8) -> Self::Output {
        Phase(self.0.wrapping_div(rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(Phase(0x02), Phase(0x01), Phase(0x01))]
    #[case(Phase(0xFE), Phase(0x7F), Phase(0x7F))]
    #[case(Phase(0x7E), Phase(0x7F), Phase(0xFF))]
    fn add(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: Phase) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase::ZERO, Phase(0x01), Phase(0x01))]
    #[case(Phase(0x01), Phase(0x02), Phase(0x01))]
    #[case(Phase(0x80), Phase(0x7F), Phase(0xFF))]
    fn sub(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: Phase) {
        assert_eq!(expected, lhs - rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase(0x02), Phase(0x01), 2)]
    #[case(Phase(0xFE), Phase(0x7F), 2)]
    #[case(Phase::ZERO, Phase(0x80), 2)]
    fn mul(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: u8) {
        assert_eq!(expected, lhs * rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase(0x01), Phase(0x02), 2)]
    #[case(Phase(0x7F), Phase(0xFE), 2)]
    #[case(Phase::ZERO, Phase(0x01), 2)]
    fn div(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: u8) {
        assert_eq!(expected, lhs / rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case(0.0, 0)]
    #[case(2.0 * PI / 256.0 * 128.0, 128)]
    #[case(2.0 * PI / 256.0 * 255.0, 255)]
    fn radian(#[case] expect: f32, #[case] value: u8) {
        approx::assert_abs_diff_eq!(expect, Phase(value).radian());
    }

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", Phase::ZERO), "0x00");
        assert_eq!(format!("{:?}", Phase(0x01)), "0x01");
        assert_eq!(format!("{:?}", Phase(0xFF)), "0xFF");
    }
}
