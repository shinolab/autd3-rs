use crate::{error::AUTDDriverError, geometry::Point3};

use super::*;
use zerocopy::{Immutable, IntoBytes};

#[bitfield_struct::bitfield(u64)]
#[derive(IntoBytes, Immutable)]
pub(crate) struct STMFocus {
    #[bits(18)]
    pub x: i32,
    #[bits(18)]
    pub y: i32,
    #[bits(18)]
    pub z: i32,
    #[bits(8)]
    pub intensity: u8,
    #[bits(2)]
    __: u8,
}

impl STMFocus {
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

        Ok(Self::new()
            .with_x(ix)
            .with_y(iy)
            .with_z(iz)
            .with_intensity(intensity_or_offset))
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
        let mut f = STMFocus::new();
        f.set_x(0b11111111111111_111111111111111111u32 as i32);
        assert_eq!(
            &[0b11111111, 0b11111111, 0b11, 0x00, 0x00, 0x00, 0x00, 0x00],
            f.as_bytes()
        );

        f.set_y(0b010101010101010101);
        assert_eq!(
            &[0b11111111, 0b11111111, 0b01010111, 0b01010101, 0b0101, 0x00, 0x00, 0x00],
            f.as_bytes()
        );

        f.set_z(0b11111111111111_101010101010101010u32 as i32);
        assert_eq!(
            &[
                0b11111111, 0b11111111, 0b01010111, 0b01010101, 0b10100101, 0b10101010, 0b101010,
                0x00
            ],
            f.as_bytes()
        );

        f.set_intensity(0xFF);
        assert_eq!(
            &[
                0b11111111, 0b11111111, 0b01010111, 0b01010101, 0b10100101, 0b10101010, 0b11101010,
                0b00111111
            ],
            f.as_bytes()
        );
    }

    #[test]
    fn test_to_fixed_num() {
        for i in FOCI_STM_FIXED_NUM_LOWER_Z..=FOCI_STM_FIXED_NUM_UPPER_Z {
            assert_eq!(
                i,
                STMFocus::to_fixed_num(i as f32 * FOCI_STM_FIXED_NUM_UNIT)
            );
        }
    }

    #[rstest::rstest]
    #[test]
    #[case(true, 1, 2, 3, 0x04)]
    #[case(true, -1, -2, -3, 0xFF)]
    #[case(
        true,
        FOCI_STM_FIXED_NUM_UPPER_X,
        FOCI_STM_FIXED_NUM_UPPER_Y,
        FOCI_STM_FIXED_NUM_UPPER_Z,
        0x01
    )]
    #[case(
        true,
        FOCI_STM_FIXED_NUM_LOWER_X,
        FOCI_STM_FIXED_NUM_LOWER_Y,
        FOCI_STM_FIXED_NUM_LOWER_Z,
        0x02
    )]
    #[case(false, FOCI_STM_FIXED_NUM_UPPER_X+1, FOCI_STM_FIXED_NUM_UPPER_Y, FOCI_STM_FIXED_NUM_UPPER_Z, 0x03)]
    #[case(false, FOCI_STM_FIXED_NUM_LOWER_X-1, FOCI_STM_FIXED_NUM_LOWER_Y, FOCI_STM_FIXED_NUM_LOWER_Z, 0x04)]
    #[case(false, FOCI_STM_FIXED_NUM_UPPER_X, FOCI_STM_FIXED_NUM_UPPER_Y+1, FOCI_STM_FIXED_NUM_UPPER_Z, 0x05)]
    #[case(false, FOCI_STM_FIXED_NUM_LOWER_X, FOCI_STM_FIXED_NUM_LOWER_Y-1, FOCI_STM_FIXED_NUM_LOWER_Z, 0x06)]
    #[case(false, FOCI_STM_FIXED_NUM_UPPER_X, FOCI_STM_FIXED_NUM_UPPER_Y, FOCI_STM_FIXED_NUM_UPPER_Z+1, 0x07)]
    #[case(false, FOCI_STM_FIXED_NUM_LOWER_X, FOCI_STM_FIXED_NUM_LOWER_Y, FOCI_STM_FIXED_NUM_LOWER_Z-1, 0x08)]
    fn test_stm_focus(
        #[case] expect: bool,
        #[case] x: i32,
        #[case] y: i32,
        #[case] z: i32,
        #[case] intensity: u8,
    ) {
        let p = STMFocus::create(
            &Point3::new(
                x as f32 * FOCI_STM_FIXED_NUM_UNIT,
                y as f32 * FOCI_STM_FIXED_NUM_UNIT,
                z as f32 * FOCI_STM_FIXED_NUM_UNIT,
            ),
            intensity,
        );
        assert_eq!(expect, p.is_ok());
        if expect {
            let p = p.unwrap();
            assert_eq!({ x }, p.x());
            assert_eq!({ y }, p.y());
            assert_eq!({ z }, p.z());
            assert_eq!(intensity, p.intensity());
        }
    }
}
