use crate::{error::AUTDDriverError, geometry::Point3};

use super::*;
use autd3_core::datagram::FirmwareLimits;
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
    #[must_use]
    fn to_fixed_num(x: f32) -> i32 {
        (x / FOCI_STM_FIXED_NUM_UNIT).round() as i32
    }

    pub(crate) fn create(
        p: &Point3,
        intensity_or_offset: u8,
        limits: &FirmwareLimits,
    ) -> Result<Self, AUTDDriverError> {
        let ix = Self::to_fixed_num(p.x);
        let iy = Self::to_fixed_num(p.y);
        let iz = Self::to_fixed_num(p.z);

        if !(limits.foci_stm_fixed_num_lower_x()..=limits.foci_stm_fixed_num_upper_x())
            .contains(&ix)
            || !(limits.foci_stm_fixed_num_lower_y()..=limits.foci_stm_fixed_num_upper_y())
                .contains(&iy)
            || !(limits.foci_stm_fixed_num_lower_z()..=limits.foci_stm_fixed_num_upper_z())
                .contains(&iz)
        {
            return Err(AUTDDriverError::FociSTMPointOutOfRange(
                p.x, p.y, p.z, *limits,
            ));
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
    use crate::firmware::driver::Driver;

    use super::*;

    #[test]
    fn size() {
        assert_eq!(8, std::mem::size_of::<STMFocus>());
    }

    #[test]
    fn bitfield() {
        let mut f = STMFocus::new();
        f.set_x(0b11111111111111_111111111111111111u32 as i32);
        assert_eq!(
            &[0b11111111, 0b11111111, 0b11, 0x00, 0x00, 0x00, 0x00, 0x00],
            f.as_bytes()
        );

        f.set_y(0b010101010101010101);
        assert_eq!(
            &[
                0b11111111, 0b11111111, 0b01010111, 0b01010101, 0b0101, 0x00, 0x00, 0x00
            ],
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

    #[rstest::fixture]
    fn limits() -> FirmwareLimits {
        super::super::super::V10.firmware_limits()
    }

    #[rstest::rstest]
    #[test]
    fn to_fixed_num(limits: FirmwareLimits) {
        (limits.foci_stm_fixed_num_lower_z()..=limits.foci_stm_fixed_num_upper_z()).for_each(|i| {
            assert_eq!(
                i,
                STMFocus::to_fixed_num(i as f32 * FOCI_STM_FIXED_NUM_UNIT)
            );
        });
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 2, 3, 0x04)]
    #[case(-1, -2, -3, 0xFF)]
    fn stm_focus(
        #[case] x: i32,
        #[case] y: i32,
        #[case] z: i32,
        #[case] intensity: u8,
        limits: FirmwareLimits,
    ) {
        let p = STMFocus::create(
            &Point3::new(
                x as f32 * FOCI_STM_FIXED_NUM_UNIT,
                y as f32 * FOCI_STM_FIXED_NUM_UNIT,
                z as f32 * FOCI_STM_FIXED_NUM_UNIT,
            ),
            intensity,
            &limits,
        );
        assert!(p.is_ok());
        let p = p.unwrap();
        assert_eq!({ x }, p.x());
        assert_eq!({ y }, p.y());
        assert_eq!({ z }, p.z());
        assert_eq!(intensity, p.intensity());
    }

    #[rstest::rstest]
    #[test]
    fn marginal(limits: FirmwareLimits) {
        let check = |expect, x, y, z| {
            let p = STMFocus::create(
                &Point3::new(
                    x as f32 * limits.foci_stm_fixed_num_unit,
                    y as f32 * limits.foci_stm_fixed_num_unit,
                    z as f32 * limits.foci_stm_fixed_num_unit,
                ),
                0xFF,
                &limits,
            );
            assert_eq!(expect, p.is_ok());
            if expect {
                let p = p.unwrap();
                assert_eq!({ x }, p.x());
                assert_eq!({ y }, p.y());
                assert_eq!({ z }, p.z());
            }
        };

        check(
            true,
            limits.foci_stm_fixed_num_lower_x(),
            limits.foci_stm_fixed_num_lower_y(),
            limits.foci_stm_fixed_num_lower_z(),
        );
        check(
            true,
            limits.foci_stm_fixed_num_upper_x(),
            limits.foci_stm_fixed_num_upper_y(),
            limits.foci_stm_fixed_num_upper_z(),
        );
        check(
            false,
            limits.foci_stm_fixed_num_lower_x() - 1,
            limits.foci_stm_fixed_num_lower_y(),
            limits.foci_stm_fixed_num_lower_z(),
        );
        check(
            false,
            limits.foci_stm_fixed_num_upper_x() + 1,
            limits.foci_stm_fixed_num_upper_y(),
            limits.foci_stm_fixed_num_upper_z(),
        );
        check(
            false,
            limits.foci_stm_fixed_num_lower_x(),
            limits.foci_stm_fixed_num_lower_y() - 1,
            limits.foci_stm_fixed_num_lower_z(),
        );
        check(
            false,
            limits.foci_stm_fixed_num_lower_x(),
            limits.foci_stm_fixed_num_upper_y() + 1,
            limits.foci_stm_fixed_num_upper_z(),
        );
        check(
            false,
            limits.foci_stm_fixed_num_lower_x(),
            limits.foci_stm_fixed_num_lower_y(),
            limits.foci_stm_fixed_num_lower_z() - 1,
        );
        check(
            false,
            limits.foci_stm_fixed_num_lower_x(),
            limits.foci_stm_fixed_num_lower_y(),
            limits.foci_stm_fixed_num_upper_z() + 1,
        );
    }
}
