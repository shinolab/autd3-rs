mod debug_type;
mod drive;
mod emit_intensity;
mod fpga_state;
mod gpio;
mod loop_behavior;
mod phase;
mod sampling_config;
mod segment;
mod silencer_target;
mod stm_focus;
mod transition_mode;

pub use debug_type::DebugType;
pub(crate) use debug_type::DebugValue;
pub use drive::Drive;
pub use emit_intensity::EmitIntensity;
pub use fpga_state::FPGAState;
pub use gpio::*;
pub use loop_behavior::LoopBehavior;
pub use phase::Phase;
pub use sampling_config::SamplingConfig;
pub use segment::Segment;
pub use silencer_target::SilencerTarget;
pub(crate) use stm_focus::STMFocus;
pub use transition_mode::*;

use crate::{
    defined::{mm, Freq},
    ethercat::DcSysTime,
};

pub const FPGA_MAIN_CLK_FREQ: Freq<u32> = Freq { freq: 10240000 };

pub const FOCI_STM_FIXED_NUM_UNIT: f32 = 0.025 * mm;
pub const FOCI_STM_FIXED_NUM_WIDTH: usize = 18;
const FOCI_STM_FIXED_NUM_UPPER: i32 = (1 << (FOCI_STM_FIXED_NUM_WIDTH - 1)) - 1;
const FOCI_STM_FIXED_NUM_LOWER: i32 = -(1 << (FOCI_STM_FIXED_NUM_WIDTH - 1));
const FOCI_STM_TR_X_MAX: i32 = 0x1AFC;
const FOCI_STM_TR_Y_MAX: i32 = 0x14A3;
pub const FOCI_STM_FIXED_NUM_UPPER_X: i32 = FOCI_STM_FIXED_NUM_UPPER;
pub const FOCI_STM_FIXED_NUM_LOWER_X: i32 = FOCI_STM_FIXED_NUM_LOWER + FOCI_STM_TR_X_MAX;
pub const FOCI_STM_FIXED_NUM_UPPER_Y: i32 = FOCI_STM_FIXED_NUM_UPPER;
pub const FOCI_STM_FIXED_NUM_LOWER_Y: i32 = FOCI_STM_FIXED_NUM_LOWER + FOCI_STM_TR_Y_MAX;
pub const FOCI_STM_FIXED_NUM_UPPER_Z: i32 = FOCI_STM_FIXED_NUM_UPPER;
pub const FOCI_STM_FIXED_NUM_LOWER_Z: i32 = FOCI_STM_FIXED_NUM_LOWER;

pub const SILENCER_STEPS_INTENSITY_DEFAULT: u32 = 10;
pub const SILENCER_STEPS_PHASE_DEFAULT: u32 = 40;

pub const MOD_BUF_SIZE_MIN: usize = 2;
pub const MOD_BUF_SIZE_MAX: usize = 32768;

pub const STM_BUF_SIZE_MIN: usize = 2;
pub const FOCI_STM_FOCI_NUM_MAX: usize = 8;
pub const FOCI_STM_BUF_SIZE_MAX: usize = 8192;
pub const GAIN_STM_BUF_SIZE_MAX: usize = 1024;

pub const PWE_BUF_SIZE: usize = 256;

pub(crate) fn ec_time_to_sys_time(time: &DcSysTime) -> u64 {
    (time.sys_time() / 3125) << 5
}
