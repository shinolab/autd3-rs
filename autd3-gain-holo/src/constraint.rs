use autd3_driver::firmware::fpga::EmitIntensity;

/// Emission constraint
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmissionConstraint {
    /// Do nothing (this is equivalent to `Clamp(EmitIntensity::MIN, EmitIntensity::MAX)`)
    DontCare,
    /// Normalize the value by dividing the maximum value
    Normalize,
    /// Normalize and then multiply by the specified value
    Multiply(f64),
    /// Set all amplitudes to the specified value
    Uniform(EmitIntensity),
    /// Clamp all amplitudes to the specified range
    Clamp(EmitIntensity, EmitIntensity),
}

impl EmissionConstraint {
    pub fn convert(&self, value: f64, max_value: f64) -> EmitIntensity {
        match self {
            EmissionConstraint::DontCare => {
                EmitIntensity::new((value * 255.).round().clamp(0., 255.) as u8)
            }
            EmissionConstraint::Normalize => {
                EmitIntensity::new((value / max_value * 255.).round() as u8)
            }
            EmissionConstraint::Multiply(v) => {
                EmitIntensity::new((value / max_value * 255. * v).round().clamp(0., 255.) as u8)
            }
            EmissionConstraint::Uniform(v) => *v,
            EmissionConstraint::Clamp(min, max) => EmitIntensity::new(
                (value * 255.)
                    .round()
                    .clamp(min.value() as f64, max.value() as f64) as u8,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(EmitIntensity::MIN, 0.0, 1.0)]
    #[case(EmitIntensity::new(128), 0.5, 1.0)]
    #[case(EmitIntensity::MAX, 1.0, 1.0)]
    #[case(EmitIntensity::MAX, 1.5, 1.0)]
    fn dont_care(#[case] expect: EmitIntensity, #[case] value: f64, #[case] max_value: f64) {
        assert_eq!(
            expect,
            EmissionConstraint::DontCare.convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(EmitIntensity::MIN, 0.0, 1.0)]
    #[case(EmitIntensity::new(128), 0.5, 1.0)]
    #[case(EmitIntensity::new(128), 1.0, 2.0)]
    #[case(EmitIntensity::new(191), 1.5, 2.0)]
    fn normalize(#[case] expect: EmitIntensity, #[case] value: f64, #[case] max_value: f64) {
        assert_eq!(
            expect,
            EmissionConstraint::Normalize.convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(EmitIntensity::MIN, 0.0, 1.0, 0.5)]
    #[case(EmitIntensity::new(64), 0.5, 1.0, 0.5)]
    #[case(EmitIntensity::new(64), 1.0, 2.0, 0.5)]
    #[case(EmitIntensity::new(96), 1.5, 2.0, 0.5)]
    fn multiply(
        #[case] expect: EmitIntensity,
        #[case] value: f64,
        #[case] max_value: f64,
        #[case] mul: f64,
    ) {
        assert_eq!(
            expect,
            EmissionConstraint::Multiply(mul).convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(EmitIntensity::MIN, 0.0, 1.0)]
    #[case(EmitIntensity::MAX, 0.0, 1.0)]
    #[case(EmitIntensity::MIN, 0.5, 1.0)]
    #[case(EmitIntensity::MAX, 0.5, 1.0)]
    #[case(EmitIntensity::MIN, 1.0, 2.0)]
    #[case(EmitIntensity::MAX, 1.0, 2.0)]
    #[case(EmitIntensity::MIN, 1.5, 2.0)]
    #[case(EmitIntensity::MAX, 1.5, 2.0)]
    fn uniform(#[case] expect: EmitIntensity, #[case] value: f64, #[case] max_value: f64) {
        assert_eq!(
            expect,
            EmissionConstraint::Uniform(expect).convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        EmitIntensity::new(64),
        0.0,
        1.0,
        EmitIntensity::new(64),
        EmitIntensity::new(192)
    )]
    #[case(
        EmitIntensity::new(128),
        0.5,
        1.0,
        EmitIntensity::new(64),
        EmitIntensity::new(192)
    )]
    #[case(
        EmitIntensity::new(192),
        1.0,
        1.0,
        EmitIntensity::new(64),
        EmitIntensity::new(192)
    )]
    #[case(
        EmitIntensity::new(192),
        1.5,
        1.0,
        EmitIntensity::new(64),
        EmitIntensity::new(192)
    )]
    fn clamp(
        #[case] expect: EmitIntensity,
        #[case] value: f64,
        #[case] max_value: f64,
        #[case] min: EmitIntensity,
        #[case] max: EmitIntensity,
    ) {
        assert_eq!(
            expect,
            EmissionConstraint::Clamp(min, max).convert(value, max_value)
        );
    }
}
