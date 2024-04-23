use crate::firmware::fpga::Drive;

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct FPGADrive {
    phase: u8,
    intensity: u8,
}

impl FPGADrive {
    pub fn set(&mut self, d: &Drive) {
        self.intensity = d.intensity().value();
        self.phase = d.phase().value();
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::firmware::fpga::{EmitIntensity, Phase};

    #[test]
    fn test_size() {
        assert_eq!(2, size_of::<FPGADrive>());
        assert_eq!(0, std::mem::offset_of!(FPGADrive, phase));
        assert_eq!(1, std::mem::offset_of!(FPGADrive, intensity));
    }

    #[rstest::rstest]
    #[test]
    #[case(Phase::new(0x00), EmitIntensity::new(0x00))]
    #[case(Phase::new(0x80), EmitIntensity::new(0xFF))]
    #[case(Phase::new(0xFF), EmitIntensity::new(0x80))]
    fn test_set(#[case] phase: Phase, #[case] intensity: EmitIntensity) {
        let mut d = FPGADrive {
            phase: 0,
            intensity: 0,
        };
        d.set(&Drive::new(phase, intensity));
        assert_eq!(phase.value(), d.phase);
        assert_eq!(intensity.value(), d.intensity);
    }
}
