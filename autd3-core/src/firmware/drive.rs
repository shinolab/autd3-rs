use super::{intensity::Intensity, phase::Phase};

use zerocopy::{Immutable, IntoBytes};

/// A container for the phase and intensity of the ultrasound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoBytes, Immutable)]
#[repr(C)]
pub struct Drive {
    /// The phase of the ultrasound.
    pub phase: Phase,
    /// The intensity of the ultrasound.
    pub intensity: Intensity,
}

impl Drive {
    /// A [`Drive`] with a phase of [`Phase::ZERO`] and an intensity of [`Intensity::MIN`].
    pub const NULL: Self = Self {
        phase: Phase::ZERO,
        intensity: Intensity::MIN,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null() {
        assert_eq!(
            Drive {
                phase: Phase::ZERO,
                intensity: Intensity::MIN
            },
            Drive::NULL
        );
    }
}
