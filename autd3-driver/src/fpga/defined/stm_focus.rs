use crate::{common::EmitIntensity, defined::float, error::AUTDInternalError};

use super::{FOCUS_STM_FIXED_NUM_LOWER, FOCUS_STM_FIXED_NUM_UNIT, FOCUS_STM_FIXED_NUM_UPPER};

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
    fn to_fixed_num(x: float) -> Result<i32, AUTDInternalError> {
        let ix = (x / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
        if !(FOCUS_STM_FIXED_NUM_LOWER..=FOCUS_STM_FIXED_NUM_UPPER).contains(&ix) {
            return Err(AUTDInternalError::FocusSTMPointOutOfRange(x));
        }
        Ok(ix)
    }

    pub fn set(
        &mut self,
        x: float,
        y: float,
        z: float,
        intensity: EmitIntensity,
    ) -> Result<(), AUTDInternalError> {
        self.set_x(Self::to_fixed_num(x)?);
        self.set_y(Self::to_fixed_num(y)?);
        self.set_z(Self::to_fixed_num(z)?);
        self.set_intensity(intensity.value());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fpga::FOCUS_STM_FIXED_NUM_WIDTH;

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

    #[rstest::rstest]
    #[test]
    #[case(Ok(1), 1)]
    #[case(Ok(-1), -1)]
    #[case(Ok((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1), (1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1)]
    #[case(Ok(-(1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1))), -(1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)))]
    #[case(Err(AUTDInternalError::FocusSTMPointOutOfRange(3276.8)), (1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1 + 1)]
    #[case(Err(AUTDInternalError::FocusSTMPointOutOfRange(-3276.8250000000003)), -(1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1)]
    fn test_to_fixed_num(#[case] expected: Result<i32, AUTDInternalError>, #[case] input: i32) {
        assert_eq!(
            expected,
            STMFocus::to_fixed_num(input as float * FOCUS_STM_FIXED_NUM_UNIT)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(1., 2., 3., 0x04)]
    #[case(-1., -2., -3., 0xFF)]
    #[case(((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as float, ((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as float, ((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as float, 0x01)]
    #[case(-(((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as float),-(((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as float),-(((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as float), 0x02)]
    fn test_stm_focus(#[case] x: float, #[case] y: float, #[case] z: float, #[case] intensity: u8) {
        let mut p = STMFocus::new();
        assert!(p
            .set(
                x * FOCUS_STM_FIXED_NUM_UNIT,
                y * FOCUS_STM_FIXED_NUM_UNIT,
                z * FOCUS_STM_FIXED_NUM_UNIT,
                EmitIntensity::new(intensity)
            )
            .is_ok());

        assert_eq!(x as i32, p.x());
        assert_eq!(y as i32, p.y());
        assert_eq!(z as i32, p.z());
        assert_eq!(intensity, p.intensity());
    }
}
