use autd3_driver::{common::EmitIntensity, defined::float};

/// Emission constraint
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmissionConstraint {
    /// Do nothing (this is equivalent to `Clamp(EmitIntensity::MIN, EmitIntensity::MAX)`)
    DontCare,
    /// Normalize the value by dividing the maximum value
    Normalize,
    /// Set all amplitudes to the specified value
    Uniform(EmitIntensity),
    /// Clamp all amplitudes to the specified range
    Clamp(EmitIntensity, EmitIntensity),
}

impl EmissionConstraint {
    pub fn convert(&self, value: float, max_value: float) -> EmitIntensity {
        match self {
            EmissionConstraint::DontCare => {
                EmitIntensity::new((value * 255.).round().clamp(0., 255.) as u8)
            }
            EmissionConstraint::Normalize => {
                EmitIntensity::new((value / max_value * 255.).round() as u8)
            }
            EmissionConstraint::Uniform(v) => *v,
            EmissionConstraint::Clamp(min, max) => EmitIntensity::new(
                (value * 255.)
                    .round()
                    .clamp(min.value() as float, max.value() as float) as u8,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_dont_care() {
        let c = EmissionConstraint::DontCare;
        assert_eq!(c.convert(0.0, 1.0), EmitIntensity::MIN);
        assert_eq!(c.convert(0.5, 1.0), EmitIntensity::new(128));
        assert_eq!(c.convert(1.0, 1.0), EmitIntensity::MAX);
        assert_eq!(c.convert(1.5, 1.0), EmitIntensity::MAX);
    }

    #[test]
    fn test_constraint_normalize() {
        let c = EmissionConstraint::Normalize;
        assert_eq!(c.convert(0.0, 1.0), EmitIntensity::MIN);
        assert_eq!(c.convert(0.5, 1.0), EmitIntensity::new(128));
        assert_eq!(c.convert(1.0, 2.0), EmitIntensity::new(128));
        assert_eq!(c.convert(1.5, 2.0), EmitIntensity::new(191));
    }

    #[test]
    fn test_constraint_uniform() {
        let c = EmissionConstraint::Uniform(EmitIntensity::new(128));
        assert_eq!(c.convert(0.0, 1.0), EmitIntensity::new(128));
        assert_eq!(c.convert(0.5, 1.0), EmitIntensity::new(128));
        assert_eq!(c.convert(1.0, 1.0), EmitIntensity::new(128));
        assert_eq!(c.convert(1.5, 1.0), EmitIntensity::new(128));
    }

    #[test]
    fn test_constraint_clamp() {
        let c = EmissionConstraint::Clamp(EmitIntensity::new(64), EmitIntensity::new(192));
        assert_eq!(c.convert(0.0, 1.0), EmitIntensity::new(64));
        assert_eq!(c.convert(0.5, 1.0), EmitIntensity::new(128));
        assert_eq!(c.convert(1.0, 1.0), EmitIntensity::new(192));
        assert_eq!(c.convert(1.5, 1.0), EmitIntensity::new(192));
    }

    #[test]
    fn test_constraint_derive() {
        let c = EmissionConstraint::Clamp(EmitIntensity::new(64), EmitIntensity::new(192));
        let c2 = c;

        assert_eq!(c, c2);
        assert_eq!(
            format!("{:?}", c),
            "Clamp(EmitIntensity { value: 64 }, EmitIntensity { value: 192 })"
        );
    }
}
