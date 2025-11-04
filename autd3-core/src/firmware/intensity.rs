/// The intensity of the ultrasound.
///
/// The arithmetic operations of [`Intensity`] are saturated.
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

impl core::ops::Add for Intensity {
    type Output = Self;

    fn add(self, rhs: Intensity) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl core::ops::AddAssign for Intensity {
    fn add_assign(&mut self, rhs: Intensity) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl core::ops::Sub for Intensity {
    type Output = Self;

    fn sub(self, rhs: Intensity) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl core::ops::SubAssign for Intensity {
    fn sub_assign(&mut self, rhs: Intensity) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case(Intensity(0x01), Intensity(0x01), 1)]
    #[case(Intensity(0x00), Intensity(0x01), 2)]
    #[case(Intensity(0x7F), Intensity(0xFF), 2)]
    fn div(#[case] expected: Intensity, #[case] target: Intensity, #[case] d: u8) {
        assert_eq!(expected, target / d);
    }

    #[rstest::rstest]
    #[case(Intensity(0x01), Intensity(0x01), 1)]
    #[case(Intensity(0x02), Intensity(0x01), 2)]
    #[case(Intensity(0xFE), Intensity(0x7F), 2)]
    #[case(Intensity(0xFF), Intensity(0x7F), 3)]
    fn mul(#[case] expected: Intensity, #[case] target: Intensity, #[case] m: u8) {
        assert_eq!(expected, target * m);
        assert_eq!(expected, m * target);
    }

    #[rstest::rstest]
    #[case(Intensity(0x02), Intensity(0x01), Intensity(0x01))]
    #[case(Intensity(0xFE), Intensity(0x7F), Intensity(0x7F))]
    #[case(Intensity(0xFF), Intensity(0x7F), Intensity(0xFF))]
    fn add(#[case] expected: Intensity, #[case] lhs: Intensity, #[case] rhs: Intensity) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest::rstest]
    #[case(Intensity(0x02), Intensity(0x01), Intensity(0x01))]
    #[case(Intensity(0xFE), Intensity(0x7F), Intensity(0x7F))]
    #[case(Intensity(0xFF), Intensity(0x7F), Intensity(0xFF))]
    fn add_assign(#[case] expected: Intensity, #[case] mut lhs: Intensity, #[case] rhs: Intensity) {
        lhs += rhs;
        assert_eq!(expected, lhs);
    }

    #[rstest::rstest]
    #[case(Intensity(0x00), Intensity(0x01), Intensity(0x01))]
    #[case(Intensity(0x01), Intensity(0x02), Intensity(0x01))]
    #[case(Intensity(0x00), Intensity(0x7F), Intensity(0xFF))]
    fn sub(#[case] expected: Intensity, #[case] lhs: Intensity, #[case] rhs: Intensity) {
        assert_eq!(expected, lhs - rhs);
    }

    #[rstest::rstest]
    #[case(Intensity(0x00), Intensity(0x01), Intensity(0x01))]
    #[case(Intensity(0x01), Intensity(0x02), Intensity(0x01))]
    #[case(Intensity(0x00), Intensity(0x7F), Intensity(0xFF))]
    fn sub_assign(#[case] expected: Intensity, #[case] mut lhs: Intensity, #[case] rhs: Intensity) {
        lhs -= rhs;
        assert_eq!(expected, lhs);
    }

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", Intensity(0x00)), "0x00");
        assert_eq!(format!("{:?}", Intensity(0x01)), "0x01");
        assert_eq!(format!("{:?}", Intensity(0xFF)), "0xFF");
    }
}
