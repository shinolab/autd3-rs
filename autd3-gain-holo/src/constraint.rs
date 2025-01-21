use autd3_core::gain::EmitIntensity;

/// Emission constraint of transducers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum EmissionConstraint {
    /// Normalize the value.
    Normalize,
    /// Normalize the value and then multiply by the given value.
    Multiply(f32),
    /// Ignore the value calculated and use the given value.
    Uniform(EmitIntensity),
    /// Clamp the value between the given values.
    Clamp(EmitIntensity, EmitIntensity),
}

impl EmissionConstraint {
    #[doc(hidden)]
    pub fn convert(&self, value: f32, max_value: f32) -> EmitIntensity {
        match self {
            EmissionConstraint::Normalize => {
                EmitIntensity((value / max_value * 255.).round() as u8)
            }
            EmissionConstraint::Multiply(v) => {
                EmitIntensity((value / max_value * 255. * v).round().clamp(0., 255.) as u8)
            }
            EmissionConstraint::Uniform(v) => *v,
            EmissionConstraint::Clamp(min, max) => EmitIntensity(
                (value * 255.)
                    .round()
                    .clamp(min.0 as f32, max.0 as f32) as u8,
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
    #[case(EmitIntensity(128), 0.5, 1.0)]
    #[case(EmitIntensity(128), 1.0, 2.0)]
    #[case(EmitIntensity(191), 1.5, 2.0)]
    #[cfg_attr(miri, ignore)]
    fn normalize(#[case] expect: EmitIntensity, #[case] value: f32, #[case] max_value: f32) {
        assert_eq!(
            expect,
            EmissionConstraint::Normalize.convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(EmitIntensity::MIN, 0.0, 1.0, 0.5)]
    #[case(EmitIntensity(64), 0.5, 1.0, 0.5)]
    #[case(EmitIntensity(64), 1.0, 2.0, 0.5)]
    #[case(EmitIntensity(96), 1.5, 2.0, 0.5)]
    #[cfg_attr(miri, ignore)]
    fn multiply(
        #[case] expect: EmitIntensity,
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
    #[case(EmitIntensity::MIN, 0.0, 1.0)]
    #[case(EmitIntensity::MAX, 0.0, 1.0)]
    #[case(EmitIntensity::MIN, 0.5, 1.0)]
    #[case(EmitIntensity::MAX, 0.5, 1.0)]
    #[case(EmitIntensity::MIN, 1.0, 2.0)]
    #[case(EmitIntensity::MAX, 1.0, 2.0)]
    #[case(EmitIntensity::MIN, 1.5, 2.0)]
    #[case(EmitIntensity::MAX, 1.5, 2.0)]
    #[cfg_attr(miri, ignore)]
    fn uniform(#[case] expect: EmitIntensity, #[case] value: f32, #[case] max_value: f32) {
        assert_eq!(
            expect,
            EmissionConstraint::Uniform(expect).convert(value, max_value)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        EmitIntensity(64),
        0.0,
        1.0,
        EmitIntensity(64),
        EmitIntensity(192)
    )]
    #[case(
        EmitIntensity(128),
        0.5,
        1.0,
        EmitIntensity(64),
        EmitIntensity(192)
    )]
    #[case(
        EmitIntensity(192),
        1.0,
        1.0,
        EmitIntensity(64),
        EmitIntensity(192)
    )]
    #[case(
        EmitIntensity(192),
        1.5,
        1.0,
        EmitIntensity(64),
        EmitIntensity(192)
    )]
    #[cfg_attr(miri, ignore)]
    fn clamp(
        #[case] expect: EmitIntensity,
        #[case] value: f32,
        #[case] max_value: f32,
        #[case] min: EmitIntensity,
        #[case] max: EmitIntensity,
    ) {
        assert_eq!(
            expect,
            EmissionConstraint::Clamp(min, max).convert(value, max_value)
        );
    }
}
