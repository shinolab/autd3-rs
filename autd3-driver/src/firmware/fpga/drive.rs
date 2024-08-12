use super::{EmitIntensity, Phase};

use derive_more::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[display("({}, {})", phase, intensity)]
#[repr(C)]
pub struct Drive {
    phase: Phase,
    intensity: EmitIntensity,
}

impl Drive {
    pub const fn new(phase: Phase, intensity: EmitIntensity) -> Self {
        Self { phase, intensity }
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
            phase: Phase::new(0),
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
        Drive::new(Phase::new(0x00), EmitIntensity::new(0x01)),
        EmitIntensity::new(0x01)
    )]
    #[case(Drive::new(Phase::new(0x01), EmitIntensity::MAX), Phase::new(0x01))]
    #[cfg_attr(miri, ignore)]
    fn from(#[case] expected: Drive, #[case] target: impl Into<Drive>) {
        assert_eq!(expected, target.into());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        EmitIntensity::new(0x00),
        Drive::new(Phase::new(0), EmitIntensity::new(0x00))
    )]
    #[case(
        EmitIntensity::new(0x01),
        Drive::new(Phase::new(0), EmitIntensity::new(0x01))
    )]
    #[case(
        EmitIntensity::new(0xFF),
        Drive::new(Phase::new(0), EmitIntensity::new(0xFF))
    )]
    #[cfg_attr(miri, ignore)]
    fn test_intensity(#[case] expected: EmitIntensity, #[case] target: Drive) {
        assert_eq!(expected, target.intensity());
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase::new(0), Drive::new(Phase::new(0), EmitIntensity::new(0x00)))]
    #[case(Phase::new(1), Drive::new(Phase::new(1), EmitIntensity::new(0x00)))]
    #[case(
        Phase::new(0xFF),
        Drive::new(Phase::new(0xFF), EmitIntensity::new(0x00))
    )]
    #[cfg_attr(miri, ignore)]
    fn test_phase(#[case] expected: Phase, #[case] target: Drive) {
        assert_eq!(expected, target.phase());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_null() {
        assert_eq!(
            Drive::new(Phase::new(0), EmitIntensity::new(0x00)),
            Drive::null()
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn display() {
        assert_eq!(
            format!("{}", Drive::new(Phase::new(0), EmitIntensity::new(0x00))),
            "(0x00, 0x00)"
        );
    }
}
