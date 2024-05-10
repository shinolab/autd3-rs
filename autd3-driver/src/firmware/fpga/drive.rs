use super::{EmitIntensity, Phase};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Drive {
    /// Phase of ultrasound
    phase: Phase,
    /// emission intensity
    intensity: EmitIntensity,
}

impl Drive {
    pub fn new(phase: impl Into<Phase>, intensity: impl Into<EmitIntensity>) -> Self {
        Self {
            phase: phase.into(),
            intensity: intensity.into(),
        }
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub const fn intensity(&self) -> EmitIntensity {
        self.intensity
    }

    pub const fn null() -> Self {
        Self {
            phase: Phase::new(0),
            intensity: EmitIntensity::MIN,
        }
    }
}

#[cfg(any(test, feature = "rand"))]
impl rand::distributions::Distribution<Drive> for rand::distributions::Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Drive {
        Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::value_0(
        EmitIntensity::new(0x00),
        Drive::new(Phase::new(0), EmitIntensity::new(0x00))
    )]
    #[case::value_1(
        EmitIntensity::new(0x01),
        Drive::new(Phase::new(0), EmitIntensity::new(0x01))
    )]
    #[case::value_ff(
        EmitIntensity::new(0xFF),
        Drive::new(Phase::new(0), EmitIntensity::new(0xFF))
    )]
    fn test_intensity(#[case] expected: EmitIntensity, #[case] target: Drive) {
        assert_eq!(expected, target.intensity(),);
    }

    #[rstest::rstest]
    #[test]
    #[case::value_0(Phase::new(0), Drive::new(Phase::new(0), EmitIntensity::new(0x00)))]
    #[case::value_1(Phase::new(1), Drive::new(Phase::new(1), EmitIntensity::new(0x00)))]
    #[case::value_ff(
        Phase::new(0xFF),
        Drive::new(Phase::new(0xFF), EmitIntensity::new(0x00))
    )]
    fn test_phase(#[case] expected: Phase, #[case] target: Drive) {
        assert_eq!(expected, target.phase(),);
    }

    #[test]
    fn test_null() {
        assert_eq!(
            Drive::new(Phase::new(0), EmitIntensity::new(0x00)),
            Drive::null()
        );
    }
}
