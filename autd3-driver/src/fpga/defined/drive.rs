use crate::common::Drive;

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
    use crate::{
        common::{EmitIntensity, Phase},
        defined::PI,
    };

    #[test]
    fn drive() {
        assert_eq!(size_of::<FPGADrive>(), 2);

        let d = FPGADrive {
            phase: 0x01,
            intensity: 0x02,
        };
        let dc = Clone::clone(&d);
        assert_eq!(d.phase, dc.phase);
        assert_eq!(d.intensity, dc.intensity);

        let mut d = [0x00u8; 2];

        unsafe {
            let s = Drive::null();
            (*(&mut d as *mut _ as *mut FPGADrive)).set(&s);
            assert_eq!(d[0], 0x00);
            assert_eq!(d[1], 0x00);

            let s = Drive::new(Phase::from_rad(PI), EmitIntensity::new(84));
            (*(&mut d as *mut _ as *mut FPGADrive)).set(&s);
            assert_eq!(d[0], 128);
            assert_eq!(d[1], 84);

            let s = Drive::new(Phase::from_rad(2.0 * PI), EmitIntensity::MAX);
            (*(&mut d as *mut _ as *mut FPGADrive)).set(&s);
            assert_eq!(d[0], 0x00);
            assert_eq!(d[1], 0xFF);

            let s = Drive::new(Phase::from_rad(3.0 * PI), EmitIntensity::MAX);
            (*(&mut d as *mut _ as *mut FPGADrive)).set(&s);
            assert_eq!(d[0], 128);
            assert_eq!(d[1], 0xFF);

            let s = Drive::new(Phase::from_rad(-PI), EmitIntensity::MIN);
            (*(&mut d as *mut _ as *mut FPGADrive)).set(&s);
            assert_eq!(d[0], 128);
            assert_eq!(d[1], 0);
        }
    }
}
