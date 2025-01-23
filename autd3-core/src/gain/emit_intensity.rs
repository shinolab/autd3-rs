use derive_more::Debug;
use zerocopy::{Immutable, IntoBytes};

/// The intensity of the ultrasound.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, IntoBytes, Immutable)]
#[debug("{:#04X}", self.0)]
#[repr(C)]
pub struct EmitIntensity(pub u8);

impl EmitIntensity {
    /// Maximum intensity.
    pub const MAX: EmitIntensity = EmitIntensity(0xFF);
    /// Minimum intensity.
    pub const MIN: EmitIntensity = EmitIntensity(0x00);
}

impl std::ops::Div<u8> for EmitIntensity {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl std::ops::Mul<u8> for EmitIntensity {
    type Output = EmitIntensity;

    fn mul(self, rhs: u8) -> Self::Output {
        EmitIntensity(self.0.saturating_mul(rhs))
    }
}

impl std::ops::Mul<EmitIntensity> for u8 {
    type Output = EmitIntensity;

    fn mul(self, rhs: EmitIntensity) -> Self::Output {
        EmitIntensity(self.saturating_mul(rhs.0))
    }
}

impl std::ops::Add<EmitIntensity> for EmitIntensity {
    type Output = Self;

    fn add(self, rhs: EmitIntensity) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl std::ops::Sub<EmitIntensity> for EmitIntensity {
    type Output = Self;

    fn sub(self, rhs: EmitIntensity) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(EmitIntensity(0x01), EmitIntensity(0x01), 1)]
    #[case::value_1_2(EmitIntensity(0x00), EmitIntensity(0x01), 2)]
    #[case::value_ff_2(EmitIntensity(0x7F), EmitIntensity(0xFF), 2)]
    fn test_div(#[case] expected: EmitIntensity, #[case] target: EmitIntensity, #[case] div: u8) {
        assert_eq!(expected, target / div);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(EmitIntensity(0x01), EmitIntensity(0x01), 1)]
    #[case::value_1_2(EmitIntensity(0x02), EmitIntensity(0x01), 2)]
    #[case::value_7f_2(EmitIntensity(0xFE), EmitIntensity(0x7F), 2)]
    #[case::value_7f_3(EmitIntensity(0xFF), EmitIntensity(0x7F), 3)]
    fn test_mul(#[case] expected: EmitIntensity, #[case] target: EmitIntensity, #[case] mul: u8) {
        assert_eq!(expected, target * mul);
        assert_eq!(expected, mul * target);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(EmitIntensity(0x02), EmitIntensity(0x01), EmitIntensity(0x01))]
    #[case::value_7f_7f(EmitIntensity(0xFE), EmitIntensity(0x7F), EmitIntensity(0x7F))]
    #[case::value_7f_ff(EmitIntensity(0xFF), EmitIntensity(0x7F), EmitIntensity(0xFF))]
    fn test_add(
        #[case] expected: EmitIntensity,
        #[case] lhs: EmitIntensity,
        #[case] rhs: EmitIntensity,
    ) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(EmitIntensity(0x00), EmitIntensity(0x01), EmitIntensity(0x01))]
    #[case::value_7f_7f(EmitIntensity(0x01), EmitIntensity(0x02), EmitIntensity(0x01))]
    #[case::value_7f_ff(EmitIntensity(0x00), EmitIntensity(0x7F), EmitIntensity(0xFF))]
    fn test_sub(
        #[case] expected: EmitIntensity,
        #[case] lhs: EmitIntensity,
        #[case] rhs: EmitIntensity,
    ) {
        assert_eq!(expected, lhs - rhs);
    }

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", EmitIntensity(0x00)), "0x00");
        assert_eq!(format!("{:?}", EmitIntensity(0x01)), "0x01");
        assert_eq!(format!("{:?}", EmitIntensity(0xFF)), "0xFF");
    }
}
