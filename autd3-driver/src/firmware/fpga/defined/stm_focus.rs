use crate::{error::AUTDInternalError, firmware::fpga::EmitIntensity};

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
    fn to_fixed_num(x: f64) -> Result<i32, AUTDInternalError> {
        let ix = (x / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
        if !(FOCUS_STM_FIXED_NUM_LOWER..=FOCUS_STM_FIXED_NUM_UPPER).contains(&ix) {
            return Err(AUTDInternalError::FocusSTMPointOutOfRange(x));
        }
        Ok(ix)
    }

    pub fn set(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
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
    use crate::firmware::fpga::FOCUS_STM_FIXED_NUM_WIDTH;

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
    #[cfg_attr(not(feature="use_meter"), case(Err(AUTDInternalError::FocusSTMPointOutOfRange(3276.8)), (1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1 + 1))]
    #[cfg_attr(not(feature="use_meter"), case(Err(AUTDInternalError::FocusSTMPointOutOfRange(-3276.8250000000003)), -(1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1))]
    #[cfg_attr(feature="use_meter", case(Err(AUTDInternalError::FocusSTMPointOutOfRange(3.2768)), (1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1 + 1))]
    #[cfg_attr(feature="use_meter", case(Err(AUTDInternalError::FocusSTMPointOutOfRange(-3.276825)), -(1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1))]
    fn test_to_fixed_num(#[case] expected: Result<i32, AUTDInternalError>, #[case] input: i32) {
        assert_eq!(
            expected,
            STMFocus::to_fixed_num(input as f64 * FOCUS_STM_FIXED_NUM_UNIT)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(1., 2., 3., 0x04)]
    #[case(-1., -2., -3., 0xFF)]
    #[case(((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as f64, ((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as f64, ((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as f64, 0x01)]
    #[case(-(((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as f64),-(((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as f64),-(((1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1) as f64), 0x02)]
    fn test_stm_focus(#[case] x: f64, #[case] y: f64, #[case] z: f64, #[case] intensity: u8) {
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
