use derive_more::Debug;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[debug("{:#04X}", self.value)]
#[repr(C)]
pub struct EmitIntensity {
    value: u8,
}

impl EmitIntensity {
    pub const MAX: EmitIntensity = EmitIntensity { value: 255 };
    pub const MIN: EmitIntensity = EmitIntensity { value: 0 };

    pub const fn new(value: u8) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> u8 {
        self.value
    }
}

impl From<u8> for EmitIntensity {
    fn from(v: u8) -> Self {
        Self::new(v)
    }
}

impl std::ops::Div<u8> for EmitIntensity {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        Self::new(self.value / rhs)
    }
}

impl std::ops::Mul<u8> for EmitIntensity {
    type Output = EmitIntensity;

    fn mul(self, rhs: u8) -> Self::Output {
        Self::Output::new(self.value.saturating_mul(rhs))
    }
}

impl std::ops::Mul<EmitIntensity> for u8 {
    type Output = EmitIntensity;

    fn mul(self, rhs: EmitIntensity) -> Self::Output {
        Self::Output::new(self.saturating_mul(rhs.value))
    }
}

impl std::ops::Add<EmitIntensity> for EmitIntensity {
    type Output = Self;

    fn add(self, rhs: EmitIntensity) -> Self::Output {
        Self::new(self.value.saturating_add(rhs.value))
    }
}

impl std::ops::Sub<EmitIntensity> for EmitIntensity {
    type Output = Self;

    fn sub(self, rhs: EmitIntensity) -> Self::Output {
        Self::new(self.value.saturating_sub(rhs.value))
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
    fn test_new(#[case] expected: u8) {
        assert_eq!(expected, EmitIntensity::new(expected).value(),);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(EmitIntensity::new(0x01), EmitIntensity::new(0x01), 1)]
    #[case::value_1_2(EmitIntensity::new(0x00), EmitIntensity::new(0x01), 2)]
    #[case::value_ff_2(EmitIntensity::new(0x7F), EmitIntensity::new(0xFF), 2)]
    #[cfg_attr(miri, ignore)]
    fn test_div(#[case] expected: EmitIntensity, #[case] target: EmitIntensity, #[case] div: u8) {
        assert_eq!(expected, target / div);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(EmitIntensity::new(0x01), EmitIntensity::new(0x01), 1)]
    #[case::value_1_2(EmitIntensity::new(0x02), EmitIntensity::new(0x01), 2)]
    #[case::value_7f_2(EmitIntensity::new(0xFE), EmitIntensity::new(0x7F), 2)]
    #[case::value_7f_3(EmitIntensity::new(0xFF), EmitIntensity::new(0x7F), 3)]
    #[cfg_attr(miri, ignore)]
    fn test_mul(#[case] expected: EmitIntensity, #[case] target: EmitIntensity, #[case] mul: u8) {
        assert_eq!(expected, target * mul);
        assert_eq!(expected, mul * target);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(
        EmitIntensity::new(0x02),
        EmitIntensity::new(0x01),
        EmitIntensity::new(0x01)
    )]
    #[case::value_7f_7f(
        EmitIntensity::new(0xFE),
        EmitIntensity::new(0x7F),
        EmitIntensity::new(0x7F)
    )]
    #[case::value_7f_ff(
        EmitIntensity::new(0xFF),
        EmitIntensity::new(0x7F),
        EmitIntensity::new(0xFF)
    )]
    #[cfg_attr(miri, ignore)]
    fn test_add(
        #[case] expected: EmitIntensity,
        #[case] lhs: EmitIntensity,
        #[case] rhs: EmitIntensity,
    ) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_1_1(
        EmitIntensity::new(0x00),
        EmitIntensity::new(0x01),
        EmitIntensity::new(0x01)
    )]
    #[case::value_7f_7f(
        EmitIntensity::new(0x01),
        EmitIntensity::new(0x02),
        EmitIntensity::new(0x01)
    )]
    #[case::value_7f_ff(
        EmitIntensity::new(0x00),
        EmitIntensity::new(0x7F),
        EmitIntensity::new(0xFF)
    )]
    #[cfg_attr(miri, ignore)]
    fn test_sub(
        #[case] expected: EmitIntensity,
        #[case] lhs: EmitIntensity,
        #[case] rhs: EmitIntensity,
    ) {
        assert_eq!(expected, lhs - rhs);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn dbg() {
        assert_eq!(format!("{:?}", EmitIntensity::new(0x00)), "0x00");
        assert_eq!(format!("{:?}", EmitIntensity::new(0x01)), "0x01");
        assert_eq!(format!("{:?}", EmitIntensity::new(0xFF)), "0xFF");
    }
}
