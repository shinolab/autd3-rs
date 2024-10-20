use crate::defined::{rad, Angle, Complex, PI};

use autd3_derive::Builder;

use derive_more::Debug;
use derive_new::new;
use nalgebra::ComplexField;
use zerocopy::{Immutable, IntoBytes};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Builder, new, IntoBytes, Immutable)]
#[repr(C)]
#[debug("{:#04X}", self.value)]
pub struct Phase {
    #[get]
    value: u8,
}

impl Phase {
    pub const ZERO: Self = Self { value: 0 };
    pub const PI: Self = Self { value: 128 };

    pub fn radian(&self) -> f32 {
        self.value as f32 / 256.0 * 2.0 * PI
    }
}

impl From<u8> for Phase {
    fn from(v: u8) -> Self {
        Self::new(v)
    }
}

impl From<Angle> for Phase {
    fn from(v: Angle) -> Self {
        Self {
            value: (((v.radian() / (2.0 * PI) * 256.0).round() as i32) & 0xFF) as _,
        }
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
        Self::Output::new(self.value.wrapping_add(rhs.value))
    }
}

impl std::ops::Sub<Phase> for Phase {
    type Output = Phase;

    fn sub(self, rhs: Phase) -> Self::Output {
        Self::Output::new(self.value.wrapping_sub(rhs.value))
    }
}

impl std::ops::Mul<u8> for Phase {
    type Output = Phase;

    fn mul(self, rhs: u8) -> Self::Output {
        Self::Output::new(self.value.wrapping_mul(rhs))
    }
}

impl std::ops::Div<u8> for Phase {
    type Output = Phase;

    fn div(self, rhs: u8) -> Self::Output {
        Self::Output::new(self.value.wrapping_div(rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(0x00)]
    #[case(0x01)]
    #[case(0xFF)]
    fn new(#[case] expected: u8) {
        assert_eq!(expected, Phase::from(expected).value());
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase::new(0x02), Phase::new(0x01), Phase::new(0x01))]
    #[case(Phase::new(0xFE), Phase::new(0x7F), Phase::new(0x7F))]
    #[case(Phase::new(0x7E), Phase::new(0x7F), Phase::new(0xFF))]
    fn add(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: Phase) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase::ZERO, Phase::new(0x01), Phase::new(0x01))]
    #[case(Phase::new(0x01), Phase::new(0x02), Phase::new(0x01))]
    #[case(Phase::new(0x80), Phase::new(0x7F), Phase::new(0xFF))]
    fn sub(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: Phase) {
        assert_eq!(expected, lhs - rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase::new(0x02), Phase::new(0x01), 2)]
    #[case(Phase::new(0xFE), Phase::new(0x7F), 2)]
    #[case(Phase::ZERO, Phase::new(0x80), 2)]
    fn mul(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: u8) {
        assert_eq!(expected, lhs * rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase::new(0x01), Phase::new(0x02), 2)]
    #[case(Phase::new(0x7F), Phase::new(0xFE), 2)]
    #[case(Phase::ZERO, Phase::new(0x01), 2)]
    fn div(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: u8) {
        assert_eq!(expected, lhs / rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case(0.0, 0)]
    #[case(2.0 * PI / 256.0 * 128.0, 128)]
    #[case(2.0 * PI / 256.0 * 255.0, 255)]
    fn radian(#[case] expect: f32, #[case] value: u8) {
        approx::assert_abs_diff_eq!(expect, Phase::new(value).radian());
    }

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", Phase::ZERO), "0x00");
        assert_eq!(format!("{:?}", Phase::new(0x01)), "0x01");
        assert_eq!(format!("{:?}", Phase::new(0xFF)), "0xFF");
    }
}
