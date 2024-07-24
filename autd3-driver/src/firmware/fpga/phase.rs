use nalgebra::ComplexField;

use crate::{
    defined::{Angle, Complex, PI},
    derive::rad,
};

use derive_more::Display;

#[derive(Clone, Copy, PartialEq, Eq, Display)]
#[display(fmt = "{:#04X}", value)]
#[repr(C)]
pub struct Phase {
    value: u8,
}

impl Phase {
    pub const PI: Self = Self { value: 128 };

    pub const fn new(value: u8) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> u8 {
        self.value
    }

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

impl std::fmt::Debug for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04X}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::value_0(0x00)]
    #[case::value_1(0x01)]
    #[case::value_ff(0xFF)]
    #[cfg_attr(miri, ignore)]
    fn new(#[case] expected: u8) {
        assert_eq!(expected, Phase::from(expected).value());
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(Phase::new(0x02), Phase::new(0x01), Phase::new(0x01))]
    #[case::value_7f_7f(Phase::new(0xFE), Phase::new(0x7F), Phase::new(0x7F))]
    #[case::value_7f_ff(Phase::new(0x7E), Phase::new(0x7F), Phase::new(0xFF))]
    #[cfg_attr(miri, ignore)]
    fn add(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: Phase) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(Phase::new(0x00), Phase::new(0x01), Phase::new(0x01))]
    #[case::value_7f_7f(Phase::new(0x01), Phase::new(0x02), Phase::new(0x01))]
    #[case::value_7f_ff(Phase::new(0x80), Phase::new(0x7F), Phase::new(0xFF))]
    #[cfg_attr(miri, ignore)]
    fn sub(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: Phase) {
        assert_eq!(expected, lhs - rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_0(0.0, 0)]
    #[case::value_1(2.0 * PI / 256.0 * 128.0, 128)]
    #[case::value_255(2.0 * PI / 256.0 * 255.0, 255)]
    #[cfg_attr(miri, ignore)]
    fn radian(#[case] expect: f32, #[case] value: u8) {
        assert_approx_eq::assert_approx_eq!(expect, Phase::new(value).radian());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn display() {
        assert_eq!(format!("{}", Phase::new(0x00)), "0x00");
        assert_eq!(format!("{}", Phase::new(0x01)), "0x01");
        assert_eq!(format!("{}", Phase::new(0xFF)), "0xFF");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn dbg() {
        assert_eq!(format!("{:?}", Phase::new(0x00)), "0x00");
        assert_eq!(format!("{:?}", Phase::new(0x01)), "0x01");
        assert_eq!(format!("{:?}", Phase::new(0xFF)), "0xFF");
    }
}
