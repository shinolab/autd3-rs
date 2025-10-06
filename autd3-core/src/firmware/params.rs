/// The minimum buffer size of modulation.
pub const MOD_BUF_SIZE_MIN: usize = 2;
/// The maximum buffer size of [`Modulation`].
///
/// [`Modulation`]: crate::modulation::Modulation
pub const MOD_BUF_SIZE_MAX: usize = 65536;

/// The minimum buffer size of STM.
pub const STM_BUF_SIZE_MIN: usize = 2;
/// The maximum buffer size of FociSTM.
pub const FOCI_STM_BUF_SIZE_MAX: usize = 65536;
/// The maximum buffer size of GainSTM.
pub const GAIN_STM_BUF_SIZE_MAX: usize = 1024;

/// The minimum number of foci per pattern in FociSTM.
pub const FOCI_STM_FOCI_NUM_MIN: usize = 1;
/// The maximum number of foci per pattern in FociSTM.
pub const FOCI_STM_FOCI_NUM_MAX: usize = 8;

/// The unit of the fixed-point number used in the FociSTM.
pub const FOCI_STM_FIXED_NUM_UNIT: f32 = 0.025 * crate::common::mm;
/// The width of the fixed-point number used in the FociSTM.
pub const FOCI_STM_FIXED_NUM_WIDTH: usize = 18;

#[doc(hidden)]
pub const FOCI_STM_TR_X_MAX: i32 = 0x1AFC;
#[doc(hidden)]
pub const FOCI_STM_TR_Y_MAX: i32 = 0x14A3;
#[doc(hidden)]
pub const FOCI_STM_FIXED_NUM_UPPER: i32 = (1 << (FOCI_STM_FIXED_NUM_WIDTH - 1)) - 1;
#[doc(hidden)]
pub const FOCI_STM_FIXED_NUM_LOWER: i32 = -(1 << (FOCI_STM_FIXED_NUM_WIDTH - 1));
#[doc(hidden)]
pub const FOCI_STM_FIXED_NUM_UPPER_X: i32 = FOCI_STM_FIXED_NUM_UPPER;
#[doc(hidden)]
pub const FOCI_STM_FIXED_NUM_LOWER_X: i32 = FOCI_STM_FIXED_NUM_LOWER + FOCI_STM_TR_X_MAX;
#[doc(hidden)]
pub const FOCI_STM_FIXED_NUM_UPPER_Y: i32 = FOCI_STM_FIXED_NUM_UPPER;
#[doc(hidden)]
pub const FOCI_STM_FIXED_NUM_LOWER_Y: i32 = FOCI_STM_FIXED_NUM_LOWER + FOCI_STM_TR_Y_MAX;
#[doc(hidden)]
pub const FOCI_STM_FIXED_NUM_UPPER_Z: i32 = FOCI_STM_FIXED_NUM_UPPER;
#[doc(hidden)]
pub const FOCI_STM_FIXED_NUM_LOWER_Z: i32 = FOCI_STM_FIXED_NUM_LOWER;
#[doc(hidden)]
pub const FOCI_STM_UPPER_X: f32 = FOCI_STM_FIXED_NUM_UPPER_X as f32 * FOCI_STM_FIXED_NUM_UNIT;
#[doc(hidden)]
pub const FOCI_STM_LOWER_X: f32 = FOCI_STM_FIXED_NUM_LOWER_X as f32 * FOCI_STM_FIXED_NUM_UNIT;
#[doc(hidden)]
pub const FOCI_STM_UPPER_Y: f32 = FOCI_STM_FIXED_NUM_UPPER_Y as f32 * FOCI_STM_FIXED_NUM_UNIT;
#[doc(hidden)]
pub const FOCI_STM_LOWER_Y: f32 = FOCI_STM_FIXED_NUM_LOWER_Y as f32 * FOCI_STM_FIXED_NUM_UNIT;
#[doc(hidden)]
pub const FOCI_STM_UPPER_Z: f32 = FOCI_STM_FIXED_NUM_UPPER_Z as f32 * FOCI_STM_FIXED_NUM_UNIT;
#[doc(hidden)]
pub const FOCI_STM_LOWER_Z: f32 = FOCI_STM_FIXED_NUM_LOWER_Z as f32 * FOCI_STM_FIXED_NUM_UNIT;

/// The ultrasound period count bits.
pub const ULTRASOUND_PERIOD_COUNT_BITS: usize = 9;

#[doc(hidden)]
pub const PWE_BUF_SIZE: usize = 256;
