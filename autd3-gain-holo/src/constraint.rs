use autd3_core::gain::Intensity;

/// Emission constraint of transducers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmissionConstraint {
    /// Normalize the value.
    Normalize,
    /// Normalize the value and then multiply by the given value.
    Multiply(f32),
    /// Ignore the value calculated and use the given value.
    Uniform(Intensity),
    /// Clamp the value between the given values.
    Clamp(Intensity, Intensity),
}

impl EmissionConstraint {
    #[doc(hidden)]
    #[must_use]
    pub fn convert(&self, value: f32, max_value: f32) -> Intensity {
        match self {
            EmissionConstraint::Normalize => Intensity((value / max_value * 255.).round() as u8),
            EmissionConstraint::Multiply(v) => {
                Intensity((value / max_value * 255. * v).round().clamp(0., 255.) as u8)
            }
            EmissionConstraint::Uniform(v) => *v,
            EmissionConstraint::Clamp(min, max) => {
                Intensity((value * 255.).round().clamp(min.0 as f32, max.0 as f32) as u8)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(Intensity::MIN, 0.0, 1.0)]
    #[case(Intensity(128), 0.5, 1.0)]
    #[case(Intensity(128), 1.0, 2.0)]
    #[case(Intensity(191), 1.5, 2.0)]
    fn normalize(#[case] expect: Intensity, #[case] value: f32, #[case] max_value: f32) {
        assert_eq!(
            expect,
            EmissionConstraint::Normalize.convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Intensity::MIN, 0.0, 1.0, 0.5)]
    #[case(Intensity(64), 0.5, 1.0, 0.5)]
    #[case(Intensity(64), 1.0, 2.0, 0.5)]
    #[case(Intensity(96), 1.5, 2.0, 0.5)]
    fn multiply(
        #[case] expect: Intensity,
        #[case] value: f32,
        #[case] max_value: f32,
        #[case] mul: f32,
    ) {
        assert_eq!(
            expect,
            EmissionConstraint::Multiply(mul).convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Intensity::MIN, 0.0, 1.0)]
    #[case(Intensity::MAX, 0.0, 1.0)]
    #[case(Intensity::MIN, 0.5, 1.0)]
    #[case(Intensity::MAX, 0.5, 1.0)]
    #[case(Intensity::MIN, 1.0, 2.0)]
    #[case(Intensity::MAX, 1.0, 2.0)]
    #[case(Intensity::MIN, 1.5, 2.0)]
    #[case(Intensity::MAX, 1.5, 2.0)]
    fn uniform(#[case] expect: Intensity, #[case] value: f32, #[case] max_value: f32) {
        assert_eq!(
            expect,
            EmissionConstraint::Uniform(expect).convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Intensity(64), 0.0, 1.0, Intensity(64), Intensity(192))]
    #[case(Intensity(128), 0.5, 1.0, Intensity(64), Intensity(192))]
    #[case(Intensity(192), 1.0, 1.0, Intensity(64), Intensity(192))]
    #[case(Intensity(192), 1.5, 1.0, Intensity(64), Intensity(192))]
    fn clamp(
        #[case] expect: Intensity,
        #[case] value: f32,
        #[case] max_value: f32,
        #[case] min: Intensity,
        #[case] max: Intensity,
    ) {
        assert_eq!(
            expect,
            EmissionConstraint::Clamp(min, max).convert(value, max_value)
        );
    }
}
