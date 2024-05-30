use crate::{error::AUTDInternalError, firmware::fpga::EmitIntensity};

use super::*;

#[bitfield_struct::bitfield(u64)]
pub struct STMFocus {
    #[bits(18)]
    pub(crate) x: i32,
    #[bits(18)]
    pub(crate) y: i32,
    #[bits(18)]
    pub(crate) z: i32,
    #[bits(8)]
    pub(crate) intensity: u8,
    #[bits(2)]
    __: u8,
}

impl STMFocus {
    fn to_fixed_num(x: f32) -> i32 {
        (x / FOCUS_STM_FIXED_NUM_UNIT).round() as i32
    }

    pub fn set(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        intensity: EmitIntensity,
    ) -> Result<(), AUTDInternalError> {
        let ix = Self::to_fixed_num(x);
        let iy = Self::to_fixed_num(y);
        let iz = Self::to_fixed_num(z);

        if !(FOCUS_STM_FIXED_NUM_LOWER_X..=FOCUS_STM_FIXED_NUM_UPPER_X).contains(&ix)
            || !(FOCUS_STM_FIXED_NUM_LOWER_Y..=FOCUS_STM_FIXED_NUM_UPPER_Y).contains(&iy)
            || !(FOCUS_STM_FIXED_NUM_LOWER_Z..=FOCUS_STM_FIXED_NUM_UPPER_Z).contains(&iz)
        {
            return Err(AUTDInternalError::FocusSTMPointOutOfRange(x, y, z));
        }

        self.set_x(ix);
        self.set_y(iy);
        self.set_z(iz);
        self.set_intensity(intensity.value());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        assert_eq!(8, std::mem::size_of::<STMFocus>());
    }

    #[test]
    fn test_bitfield() {
        let mut d: u64 = 0;
        unsafe {
            (*(&mut d as *mut _ as *mut STMFocus))
                .set_x(0b11111111111111_111111111111111111u32 as i32);
            assert_eq!(0b111111111111111111, d);
            (*(&mut d as *mut _ as *mut STMFocus)).set_y(0b010101010101010101);
            assert_eq!(0b010101010101010101_111111111111111111, d);
            (*(&mut d as *mut _ as *mut STMFocus))
                .set_z(0b11111111111111_101010101010101010u32 as i32);
            assert_eq!(
                0b101010101010101010_010101010101010101_111111111111111111,
                d
            );
            (*(&mut d as *mut _ as *mut STMFocus)).set_intensity(0xFF);
            assert_eq!(
                0b11111111_101010101010101010_010101010101010101_111111111111111111,
                d
            );
        }
    }

    #[test]
    fn test_to_fixed_num() {
        for i in FOCUS_STM_FIXED_NUM_LOWER_Z..=FOCUS_STM_FIXED_NUM_UPPER_Z {
            assert_eq!(
                i,
                STMFocus::to_fixed_num(i as f32 * FOCUS_STM_FIXED_NUM_UNIT)
            );
        }
    }

    #[rstest::rstest]
    #[test]
    #[case(true, 1, 2, 3, 0x04)]
    #[case(true, -1, -2, -3, 0xFF)]
    #[case(
        true,
        FOCUS_STM_FIXED_NUM_UPPER_X,
        FOCUS_STM_FIXED_NUM_UPPER_Y,
        FOCUS_STM_FIXED_NUM_UPPER_Z,
        0x01
    )]
    #[case(
        true,
        FOCUS_STM_FIXED_NUM_LOWER_X,
        FOCUS_STM_FIXED_NUM_LOWER_Y,
        FOCUS_STM_FIXED_NUM_LOWER_Z,
        0x02
    )]
    #[case(false, FOCUS_STM_FIXED_NUM_UPPER_X+1, FOCUS_STM_FIXED_NUM_UPPER_Y, FOCUS_STM_FIXED_NUM_UPPER_Z, 0x03)]
    #[case(false, FOCUS_STM_FIXED_NUM_LOWER_X-1, FOCUS_STM_FIXED_NUM_LOWER_Y, FOCUS_STM_FIXED_NUM_LOWER_Z, 0x04)]
    #[case(false, FOCUS_STM_FIXED_NUM_UPPER_X, FOCUS_STM_FIXED_NUM_UPPER_Y+1, FOCUS_STM_FIXED_NUM_UPPER_Z, 0x05)]
    #[case(false, FOCUS_STM_FIXED_NUM_LOWER_X, FOCUS_STM_FIXED_NUM_LOWER_Y-1, FOCUS_STM_FIXED_NUM_LOWER_Z, 0x06)]
    #[case(false, FOCUS_STM_FIXED_NUM_UPPER_X, FOCUS_STM_FIXED_NUM_UPPER_Y, FOCUS_STM_FIXED_NUM_UPPER_Z+1, 0x07)]
    #[case(false, FOCUS_STM_FIXED_NUM_LOWER_X, FOCUS_STM_FIXED_NUM_LOWER_Y, FOCUS_STM_FIXED_NUM_LOWER_Z-1, 0x08)]
    fn test_stm_focus(
        #[case] expect: bool,
        #[case] x: i32,
        #[case] y: i32,
        #[case] z: i32,
        #[case] intensity: u8,
    ) {
        let mut p = STMFocus::new();
        assert_eq!(
            expect,
            p.set(
                x as f32 * FOCUS_STM_FIXED_NUM_UNIT,
                y as f32 * FOCUS_STM_FIXED_NUM_UNIT,
                z as f32 * FOCUS_STM_FIXED_NUM_UNIT,
                EmitIntensity::new(intensity)
            )
            .is_ok()
        );
        if expect {
            assert_eq!({ x }, p.x());
            assert_eq!({ y }, p.y());
            assert_eq!({ z }, p.z());
            assert_eq!(intensity, p.intensity());
        }
    }
}
