mod debug_type;
mod fpga_state;
mod silencer_target;
mod stm_focus;

pub use autd3_core::{
    datagram::{GPIOIn, GPIOOut, Segment, TransitionMode, TRANSITION_MODE_NONE},
    gain::{Drive, EmitIntensity, Phase},
    modulation::{LoopBehavior, SamplingConfig},
};

pub use debug_type::DebugType;
pub(crate) use debug_type::DebugValue;
pub use fpga_state::FPGAState;
pub use silencer_target::SilencerTarget;
pub(crate) use stm_focus::STMFocus;

use crate::{defined::mm, ethercat::DcSysTime};

/// The unit of the fixed-point number used in the [`FociSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
pub const FOCI_STM_FIXED_NUM_UNIT: f32 = 0.025 * mm;
const FOCI_STM_FIXED_NUM_WIDTH: usize = 18;
const FOCI_STM_FIXED_NUM_UPPER: i32 = (1 << (FOCI_STM_FIXED_NUM_WIDTH - 1)) - 1;
const FOCI_STM_FIXED_NUM_LOWER: i32 = -(1 << (FOCI_STM_FIXED_NUM_WIDTH - 1));
const FOCI_STM_TR_X_MAX: i32 = 0x1AFC;
const FOCI_STM_TR_Y_MAX: i32 = 0x14A3;
pub(crate) const FOCI_STM_FIXED_NUM_UPPER_X: i32 = FOCI_STM_FIXED_NUM_UPPER;
pub(crate) const FOCI_STM_FIXED_NUM_LOWER_X: i32 = FOCI_STM_FIXED_NUM_LOWER + FOCI_STM_TR_X_MAX;
pub(crate) const FOCI_STM_FIXED_NUM_UPPER_Y: i32 = FOCI_STM_FIXED_NUM_UPPER;
pub(crate) const FOCI_STM_FIXED_NUM_LOWER_Y: i32 = FOCI_STM_FIXED_NUM_LOWER + FOCI_STM_TR_Y_MAX;
pub(crate) const FOCI_STM_FIXED_NUM_UPPER_Z: i32 = FOCI_STM_FIXED_NUM_UPPER;
pub(crate) const FOCI_STM_FIXED_NUM_LOWER_Z: i32 = FOCI_STM_FIXED_NUM_LOWER;

#[doc(hidden)]
pub const SILENCER_STEPS_INTENSITY_DEFAULT: u16 = 10;
#[doc(hidden)]
pub const SILENCER_STEPS_PHASE_DEFAULT: u16 = 40;

/// The minimum buffer size of [`Modulation`].
///
/// [`Modulation`]: crate::datagram::Modulation
pub const MOD_BUF_SIZE_MIN: usize = 2;
/// The maximum buffer size of [`Modulation`].
///
/// [`Modulation`]: crate::datagram::Modulation
pub const MOD_BUF_SIZE_MAX: usize = 32768;

/// The minimum buffer size of [`FociSTM`] and [`GainSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
/// [`GainSTM`]: crate::datagram::GainSTM
pub const STM_BUF_SIZE_MIN: usize = 2;
/// The maximum number of foci.
pub const FOCI_STM_FOCI_NUM_MAX: usize = 8;
/// The maximum buffer size of [`FociSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
pub const FOCI_STM_BUF_SIZE_MAX: usize = 8192;
/// The maximum buffer size of [`GainSTM`].
///
/// [`GainSTM`]: crate::datagram::GainSTM
pub const GAIN_STM_BUF_SIZE_MAX: usize = 1024;

#[doc(hidden)]
pub const PWE_BUF_SIZE: usize = 256;

pub(crate) fn ec_time_to_sys_time(time: &DcSysTime) -> u64 {
    (time.sys_time() / 3125) << 5
}
