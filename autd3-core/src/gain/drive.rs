use autd3_derive::Builder;

use super::{emit_intensity::EmitIntensity, phase::Phase};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

/// A container for the phase and intensity of the ultrasound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Builder, new, IntoBytes, Immutable)]
#[repr(C)]
pub struct Drive {
    #[get]
    /// The phase of the ultrasound.
    phase: Phase,
    #[get]
    /// The intensity of the ultrasound.
    intensity: EmitIntensity,
}

impl Drive {
    /// A [`Drive`] with a phase of [`Phase::ZERO`] and an intensity of [`EmitIntensity::MIN`].
    pub const NULL: Self = Self {
        phase: Phase::ZERO,
        intensity: EmitIntensity::MIN,
    };
}

impl From<(Phase, EmitIntensity)> for Drive {
    fn from((phase, intensity): (Phase, EmitIntensity)) -> Self {
        Self { phase, intensity }
    }
}

impl From<(EmitIntensity, Phase)> for Drive {
    fn from((intensity, phase): (EmitIntensity, Phase)) -> Self {
        Self { phase, intensity }
    }
}

impl From<EmitIntensity> for Drive {
    fn from(intensity: EmitIntensity) -> Self {
        Self {
            phase: Phase::ZERO,
            intensity,
        }
    }
}

impl From<Phase> for Drive {
    fn from(phase: Phase) -> Self {
        Self {
            phase,
            intensity: EmitIntensity::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(
        Drive::new(Phase::new(0x01), EmitIntensity::new(0x02)),
        (Phase::new(0x01), EmitIntensity::new(0x02))
    )]
    #[case(
        Drive::new(Phase::new(0x01), EmitIntensity::new(0x02)),
        (EmitIntensity::new(0x02), Phase::new(0x01))
    )]
    #[case(
        Drive::new(Phase::ZERO, EmitIntensity::new(0x01)),
        EmitIntensity::new(0x01)
    )]
    #[case(Drive::new(Phase::new(0x01), EmitIntensity::MAX), Phase::new(0x01))]
    fn from(#[case] expected: Drive, #[case] target: impl Into<Drive>) {
        assert_eq!(expected, target.into());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        EmitIntensity::new(0x00),
        Drive::new(Phase::ZERO, EmitIntensity::new(0x00))
    )]
    #[case(
        EmitIntensity::new(0x01),
        Drive::new(Phase::ZERO, EmitIntensity::new(0x01))
    )]
    #[case(
        EmitIntensity::new(0xFF),
        Drive::new(Phase::ZERO, EmitIntensity::new(0xFF))
    )]
    fn test_intensity(#[case] expected: EmitIntensity, #[case] target: Drive) {
        assert_eq!(expected, target.intensity());
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase::ZERO, Drive::new(Phase::ZERO, EmitIntensity::new(0x00)))]
    #[case(Phase::new(1), Drive::new(Phase::new(1), EmitIntensity::new(0x00)))]
    #[case(
        Phase::new(0xFF),
        Drive::new(Phase::new(0xFF), EmitIntensity::new(0x00))
    )]
    fn test_phase(#[case] expected: Phase, #[case] target: Drive) {
        assert_eq!(expected, target.phase());
    }

    #[test]
    fn test_null() {
        assert_eq!(
            Drive::new(Phase::ZERO, EmitIntensity::new(0x00)),
            Drive::NULL
        );
    }
}
