use crate::{error::AUTDDriverError, geometry::Point3};

use autd3_core::firmware::{
    FOCI_STM_FIXED_NUM_LOWER_X, FOCI_STM_FIXED_NUM_LOWER_Y, FOCI_STM_FIXED_NUM_LOWER_Z,
    FOCI_STM_FIXED_NUM_UNIT, FOCI_STM_FIXED_NUM_UPPER_X, FOCI_STM_FIXED_NUM_UPPER_Y,
    FOCI_STM_FIXED_NUM_UPPER_Z,
};

#[allow(dead_code)]
pub(crate) struct STMFocus(u64);

impl STMFocus {
    const fn new(x: i32, y: i32, z: i32, intensity: u8) -> Self {
        let x = x as u64 & 0x3_FFFF;
        let y = (y as u64 & 0x3_FFFF) << 18;
        let z = (z as u64 & 0x3_FFFF) << 36;
        let intensity = (intensity as u64 & 0xFF) << 54;
        Self(x | y | z | intensity)
    }
}

impl STMFocus {
    #[must_use]
    fn to_fixed_num(x: f32) -> i32 {
        (x / FOCI_STM_FIXED_NUM_UNIT).round() as i32
    }

    pub(crate) fn create(p: &Point3, intensity_or_offset: u8) -> Result<Self, AUTDDriverError> {
        let ix = Self::to_fixed_num(p.x);
        let iy = Self::to_fixed_num(p.y);
        let iz = Self::to_fixed_num(p.z);

        if !(FOCI_STM_FIXED_NUM_LOWER_X..=FOCI_STM_FIXED_NUM_UPPER_X).contains(&ix)
            || !(FOCI_STM_FIXED_NUM_LOWER_Y..=FOCI_STM_FIXED_NUM_UPPER_Y).contains(&iy)
            || !(FOCI_STM_FIXED_NUM_LOWER_Z..=FOCI_STM_FIXED_NUM_UPPER_Z).contains(&iz)
        {
            return Err(AUTDDriverError::FociSTMPointOutOfRange(p.x, p.y, p.z));
        }

        Ok(Self::new(ix, iy, iz, intensity_or_offset))
    }

    #[cfg(test)]
    #[doc(hidden)]
    pub fn as_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size() {
        assert_eq!(8, std::mem::size_of::<STMFocus>());
    }

    #[test]
    fn bitfield() {
        let f = STMFocus::new(0b11111111111111_111111111111111111u32 as i32, 0, 0, 0);
        assert_eq!(
            [0b11111111, 0b11111111, 0b11, 0x00, 0x00, 0x00, 0x00, 0x00],
            f.as_bytes()
        );

        let f = STMFocus::new(
            0b11111111111111_111111111111111111u32 as i32,
            0b010101010101010101,
            0,
            0,
        );
        assert_eq!(
            [
                0b11111111, 0b11111111, 0b01010111, 0b01010101, 0b0101, 0x00, 0x00, 0x00
            ],
            f.as_bytes()
        );

        let f = STMFocus::new(
            0b11111111111111_111111111111111111u32 as i32,
            0b010101010101010101,
            0b11111111111111_101010101010101010u32 as i32,
            0,
        );
        assert_eq!(
            [
                0b11111111, 0b11111111, 0b01010111, 0b01010101, 0b10100101, 0b10101010, 0b101010,
                0x00
            ],
            f.as_bytes()
        );

        let f = STMFocus::new(
            0b11111111111111_111111111111111111u32 as i32,
            0b010101010101010101,
            0b11111111111111_101010101010101010u32 as i32,
            0xFF,
        );
        assert_eq!(
            [
                0b11111111, 0b11111111, 0b01010111, 0b01010101, 0b10100101, 0b10101010, 0b11101010,
                0b00111111
            ],
            f.as_bytes()
        );
    }

    #[rstest::rstest]
    fn to_fixed_num() {
        (FOCI_STM_FIXED_NUM_LOWER_Z..=FOCI_STM_FIXED_NUM_UPPER_Z).for_each(|i| {
            assert_eq!(
                i,
                STMFocus::to_fixed_num(i as f32 * FOCI_STM_FIXED_NUM_UNIT)
            );
        });
    }

    #[rstest::rstest]
    #[case(1, 2, 3, 0x04)]
    #[case(-1, -2, -3, 0xFF)]
    fn stm_focus(#[case] x: i32, #[case] y: i32, #[case] z: i32, #[case] intensity: u8) {
        let p = STMFocus::create(
            &Point3::new(
                x as f32 * FOCI_STM_FIXED_NUM_UNIT,
                y as f32 * FOCI_STM_FIXED_NUM_UNIT,
                z as f32 * FOCI_STM_FIXED_NUM_UNIT,
            ),
            intensity,
        );
        assert!(p.is_ok());
    }

    #[rstest::rstest]
    fn marginal() {
        let check = |expect, x, y, z| {
            let p = STMFocus::create(
                &Point3::new(
                    x as f32 * FOCI_STM_FIXED_NUM_UNIT,
                    y as f32 * FOCI_STM_FIXED_NUM_UNIT,
                    z as f32 * FOCI_STM_FIXED_NUM_UNIT,
                ),
                0xFF,
            );
            assert_eq!(expect, p.is_ok());
        };

        check(
            true,
            FOCI_STM_FIXED_NUM_LOWER_X,
            FOCI_STM_FIXED_NUM_LOWER_Y,
            FOCI_STM_FIXED_NUM_LOWER_Z,
        );
        check(
            true,
            FOCI_STM_FIXED_NUM_UPPER_X,
            FOCI_STM_FIXED_NUM_UPPER_Y,
            FOCI_STM_FIXED_NUM_UPPER_Z,
        );
        check(
            false,
            FOCI_STM_FIXED_NUM_LOWER_X - 1,
            FOCI_STM_FIXED_NUM_LOWER_Y,
            FOCI_STM_FIXED_NUM_LOWER_Z,
        );
        check(
            false,
            FOCI_STM_FIXED_NUM_UPPER_X + 1,
            FOCI_STM_FIXED_NUM_UPPER_Y,
            FOCI_STM_FIXED_NUM_UPPER_Z,
        );
        check(
            false,
            FOCI_STM_FIXED_NUM_LOWER_X,
            FOCI_STM_FIXED_NUM_LOWER_Y - 1,
            FOCI_STM_FIXED_NUM_LOWER_Z,
        );
        check(
            false,
            FOCI_STM_FIXED_NUM_UPPER_X,
            FOCI_STM_FIXED_NUM_UPPER_Y + 1,
            FOCI_STM_FIXED_NUM_UPPER_Z,
        );
        check(
            false,
            FOCI_STM_FIXED_NUM_LOWER_X,
            FOCI_STM_FIXED_NUM_LOWER_Y,
            FOCI_STM_FIXED_NUM_LOWER_Z - 1,
        );
        check(
            false,
            FOCI_STM_FIXED_NUM_UPPER_X,
            FOCI_STM_FIXED_NUM_UPPER_Y,
            FOCI_STM_FIXED_NUM_UPPER_Z + 1,
        );
    }
}
