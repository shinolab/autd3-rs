use nalgebra::ComplexField;

use crate::defined::{Complex, PI};

pub struct Rad;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Phase {
    value: u8,
}

impl Phase {
    pub const fn new(value: u8) -> Self {
        Self { value }
    }

    pub fn from_rad(phase: f64) -> Self {
        Self {
            value: (((phase / (2.0 * PI) * 256.0).round() as i32) & 0xFF) as _,
        }
    }

    pub const fn value(&self) -> u8 {
        self.value
    }

    pub fn radian(&self) -> f64 {
        self.value as f64 / 256.0 * 2.0 * PI
    }
}

impl From<u8> for Phase {
    fn from(v: u8) -> Self {
        Self::new(v)
    }
}

impl From<f64> for Phase {
    fn from(v: f64) -> Self {
        Self::from_rad(v)
    }
}

impl From<Complex> for Phase {
    fn from(v: Complex) -> Self {
        Self::from_rad(v.argument())
    }
}

impl std::ops::Mul<Rad> for f64 {
    type Output = Phase;

    fn mul(self, _rhs: Rad) -> Self::Output {
        Self::Output::from_rad(self)
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

#[cfg(any(test, feature = "rand"))]
impl rand::distributions::Distribution<Phase> for rand::distributions::Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Phase {
        Phase::new(rng.gen())
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
    fn test_new(#[case] expected: u8) {
        assert_eq!(expected, Phase::from(expected).value());
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(Phase::new(0x02), Phase::new(0x01), Phase::new(0x01))]
    #[case::value_7f_7f(Phase::new(0xFE), Phase::new(0x7F), Phase::new(0x7F))]
    #[case::value_7f_ff(Phase::new(0x7E), Phase::new(0x7F), Phase::new(0xFF))]
    fn test_add(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: Phase) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(Phase::new(0x00), Phase::new(0x01), Phase::new(0x01))]
    #[case::value_7f_7f(Phase::new(0x01), Phase::new(0x02), Phase::new(0x01))]
    #[case::value_7f_ff(Phase::new(0x80), Phase::new(0x7F), Phase::new(0xFF))]
    fn test_sub(#[case] expected: Phase, #[case] lhs: Phase, #[case] rhs: Phase) {
        assert_eq!(expected, lhs - rhs);
    }
}
