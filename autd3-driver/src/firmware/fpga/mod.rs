use crate::defined::METER;

mod debug_type;
mod drive;
mod emit_intensity;
mod fpga_drive;
mod fpga_state;
mod loop_behavior;
mod phase;
pub mod sampling_config;
mod segment;
mod stm_focus;
mod transition_mode;

pub(crate) use fpga_drive::FPGADrive;
pub(crate) use stm_focus::STMFocus;

pub use debug_type::DebugType;
pub use drive::Drive;
pub use emit_intensity::EmitIntensity;
pub use fpga_state::FPGAState;
pub use loop_behavior::LoopBehavior;
pub use phase::{Phase, Rad};
pub use sampling_config::SamplingConfiguration;
pub use segment::Segment;
pub use transition_mode::TransitionMode;

#[cfg(feature = "variable_freq")]
static ULTRASOUND_FREQ: std::sync::RwLock<u32> = std::sync::RwLock::new(40000);
#[cfg(not(feature = "variable_freq"))]
const ULTRASOUND_FREQ: u32 = 40000;

#[cfg(feature = "variable_freq")]
pub fn set_ultrasound_freq(freq: u32) {
    *ULTRASOUND_FREQ.write().unwrap() = freq;
}

#[cfg(feature = "variable_freq")]
pub fn ultrasound_freq() -> u32 {
    *ULTRASOUND_FREQ.read().unwrap()
}
#[cfg(not(feature = "variable_freq"))]
pub const fn ultrasound_freq() -> u32 {
    ULTRASOUND_FREQ
}

pub const ULTRASOUND_PERIOD: u32 = 512;

#[const_fn::const_fn(cfg(not(feature = "variable_freq")))]
pub const fn fpga_clk_freq() -> u32 {
    ultrasound_freq() * ULTRASOUND_PERIOD
}

pub const FREQ_40K: u32 = 40000;

pub const FOCUS_STM_FIXED_NUM_UNIT: f64 = 0.025e-3 * METER;
pub const FOCUS_STM_FIXED_NUM_WIDTH: usize = 18;
const FOCUS_STM_FIXED_NUM_UPPER: i32 = (1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1;
const FOCUS_STM_FIXED_NUM_LOWER: i32 = -(1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1));
const FOCUS_STM_TR_X_MAX: i32 = 0x1AFC;
const FOCUS_STM_TR_Y_MAX: i32 = 0x14A3;
pub const FOCUS_STM_FIXED_NUM_UPPER_X: i32 = FOCUS_STM_FIXED_NUM_UPPER;
pub const FOCUS_STM_FIXED_NUM_LOWER_X: i32 = FOCUS_STM_FIXED_NUM_LOWER + FOCUS_STM_TR_X_MAX;
pub const FOCUS_STM_FIXED_NUM_UPPER_Y: i32 = FOCUS_STM_FIXED_NUM_UPPER;
pub const FOCUS_STM_FIXED_NUM_LOWER_Y: i32 = FOCUS_STM_FIXED_NUM_LOWER + FOCUS_STM_TR_Y_MAX;
pub const FOCUS_STM_FIXED_NUM_UPPER_Z: i32 = FOCUS_STM_FIXED_NUM_UPPER;
pub const FOCUS_STM_FIXED_NUM_LOWER_Z: i32 = FOCUS_STM_FIXED_NUM_LOWER;

pub const SILENCER_VALUE_MIN: u16 = 1;
pub const SILENCER_VALUE_MAX: u16 = 0xFFFF;
pub const SILENCER_STEPS_INTENSITY_DEFAULT: u16 = 10;
pub const SILENCER_STEPS_PHASE_DEFAULT: u16 = 40;

pub const SAMPLING_FREQ_DIV_MIN: u32 = 512;
pub const SAMPLING_FREQ_DIV_MAX: u32 = u32::MAX;

pub const MOD_BUF_SIZE_MIN: usize = 2;
pub const MOD_BUF_SIZE_MAX: usize = 32768;

pub const STM_BUF_SIZE_MIN: usize = 2;
pub const FOCUS_STM_BUF_SIZE_MAX: usize = 65536;
pub const GAIN_STM_BUF_SIZE_MAX: usize = 1024;

pub const PWE_BUF_SIZE: usize = 65536;
