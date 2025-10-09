/// The intensity of the ultrasound.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Intensity(pub u8);

impl core::fmt::Debug for Intensity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:02X}", self.0)
    }
}

impl Intensity {
    /// Maximum intensity.
    pub const MAX: Intensity = Intensity(0xFF);
    /// Minimum intensity.
    pub const MIN: Intensity = Intensity(0x00);
}

impl core::ops::Div<u8> for Intensity {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl core::ops::Mul<u8> for Intensity {
    type Output = Intensity;

    fn mul(self, rhs: u8) -> Self::Output {
        Intensity(self.0.saturating_mul(rhs))
    }
}

impl core::ops::Mul<Intensity> for u8 {
    type Output = Intensity;

    fn mul(self, rhs: Intensity) -> Self::Output {
        Intensity(self.saturating_mul(rhs.0))
    }
}

impl core::ops::Add<Intensity> for Intensity {
    type Output = Self;

    fn add(self, rhs: Intensity) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl core::ops::Sub<Intensity> for Intensity {
    type Output = Self;

    fn sub(self, rhs: Intensity) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::value_1_1(Intensity(0x01), Intensity(0x01), 1)]
    #[case::value_1_2(Intensity(0x00), Intensity(0x01), 2)]
    #[case::value_ff_2(Intensity(0x7F), Intensity(0xFF), 2)]
    fn test_div(#[case] expected: Intensity, #[case] target: Intensity, #[case] div: u8) {
        assert_eq!(expected, target / div);
    }

    #[rstest::rstest]
    #[case::value_1_1(Intensity(0x01), Intensity(0x01), 1)]
    #[case::value_1_2(Intensity(0x02), Intensity(0x01), 2)]
    #[case::value_7f_2(Intensity(0xFE), Intensity(0x7F), 2)]
    #[case::value_7f_3(Intensity(0xFF), Intensity(0x7F), 3)]
    fn test_mul(#[case] expected: Intensity, #[case] target: Intensity, #[case] mul: u8) {
        assert_eq!(expected, target * mul);
        assert_eq!(expected, mul * target);
    }

    #[rstest::rstest]
    #[case::value_1_1(Intensity(0x02), Intensity(0x01), Intensity(0x01))]
    #[case::value_7f_7f(Intensity(0xFE), Intensity(0x7F), Intensity(0x7F))]
    #[case::value_7f_ff(Intensity(0xFF), Intensity(0x7F), Intensity(0xFF))]
    fn test_add(#[case] expected: Intensity, #[case] lhs: Intensity, #[case] rhs: Intensity) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest::rstest]
    #[case::value_1_1(Intensity(0x00), Intensity(0x01), Intensity(0x01))]
    #[case::value_7f_7f(Intensity(0x01), Intensity(0x02), Intensity(0x01))]
    #[case::value_7f_ff(Intensity(0x00), Intensity(0x7F), Intensity(0xFF))]
    fn test_sub(#[case] expected: Intensity, #[case] lhs: Intensity, #[case] rhs: Intensity) {
        assert_eq!(expected, lhs - rhs);
    }

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", Intensity(0x00)), "0x00");
        assert_eq!(format!("{:?}", Intensity(0x01)), "0x01");
        assert_eq!(format!("{:?}", Intensity(0xFF)), "0xFF");
    }
}
