use crate::common::{FOCI_STM_TR_X_MAX, FOCI_STM_TR_Y_MAX};

/// Limits of the AUTD3 firmware.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FirmwareLimits {
    /// Maximum modulation buffer size.
    pub mod_buf_size_max: u32,
    /// Maximum GainSTM buffer size.
    pub gain_stm_buf_size_max: u32,
    /// Maximum FociSTM buffer size.
    pub foci_stm_buf_size_max: u32,
    /// Maximum number of foci per pattern in FociSTM.
    pub num_foci_max: u32,
    /// Fixed-point number unit for FociSTM.
    pub foci_stm_fixed_num_unit: f32,
    /// Width of the fixed-point number for FociSTM.
    pub foci_stm_fixed_num_width: u32,
}

#[doc(hidden)]
impl FirmwareLimits {
    pub fn unused() -> Self {
        Self {
            mod_buf_size_max: 0,
            gain_stm_buf_size_max: 0,
            foci_stm_buf_size_max: 0,
            num_foci_max: 0,
            foci_stm_fixed_num_unit: 0.0,
            foci_stm_fixed_num_width: 0,
        }
    }

    pub const fn foci_stm_fixed_num_upper(&self) -> i32 {
        (1 << (self.foci_stm_fixed_num_width - 1)) - 1
    }

    pub const fn foci_stm_fixed_num_lower(&self) -> i32 {
        -(1 << (self.foci_stm_fixed_num_width - 1))
    }

    pub const fn foci_stm_fixed_num_upper_x(&self) -> i32 {
        self.foci_stm_fixed_num_upper()
    }

    pub const fn foci_stm_fixed_num_lower_x(&self) -> i32 {
        self.foci_stm_fixed_num_lower() + FOCI_STM_TR_X_MAX
    }

    pub const fn foci_stm_fixed_num_upper_y(&self) -> i32 {
        self.foci_stm_fixed_num_upper()
    }

    pub const fn foci_stm_fixed_num_lower_y(&self) -> i32 {
        self.foci_stm_fixed_num_lower() + FOCI_STM_TR_Y_MAX
    }

    pub const fn foci_stm_fixed_num_upper_z(&self) -> i32 {
        self.foci_stm_fixed_num_upper()
    }

    pub const fn foci_stm_fixed_num_lower_z(&self) -> i32 {
        self.foci_stm_fixed_num_lower()
    }

    pub const fn foci_stm_upper_x(&self) -> f32 {
        self.foci_stm_fixed_num_upper_x() as f32 * self.foci_stm_fixed_num_unit
    }

    pub const fn foci_stm_lower_x(&self) -> f32 {
        self.foci_stm_fixed_num_lower_x() as f32 * self.foci_stm_fixed_num_unit
    }

    pub const fn foci_stm_upper_y(&self) -> f32 {
        self.foci_stm_fixed_num_upper_y() as f32 * self.foci_stm_fixed_num_unit
    }

    pub const fn foci_stm_lower_y(&self) -> f32 {
        self.foci_stm_fixed_num_lower_y() as f32 * self.foci_stm_fixed_num_unit
    }

    pub const fn foci_stm_upper_z(&self) -> f32 {
        self.foci_stm_fixed_num_upper_z() as f32 * self.foci_stm_fixed_num_unit
    }

    pub const fn foci_stm_lower_z(&self) -> f32 {
        self.foci_stm_fixed_num_lower_z() as f32 * self.foci_stm_fixed_num_unit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foci_stm_limits() {
        let limits = FirmwareLimits {
            foci_stm_fixed_num_unit: 0.025,
            foci_stm_fixed_num_width: 18,
            ..FirmwareLimits::unused()
        };

        assert_eq!(3276.7751, limits.foci_stm_upper_x());
        assert_eq!(-3104.1, limits.foci_stm_lower_x());
        assert_eq!(3276.7751, limits.foci_stm_upper_y());
        assert_eq!(-3144.725, limits.foci_stm_lower_y());
        assert_eq!(3276.7751, limits.foci_stm_upper_z());
        assert_eq!(-3276.8, limits.foci_stm_lower_z());
    }
}
