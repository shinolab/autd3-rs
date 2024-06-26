mod debug_type;
mod drive;
mod emit_intensity;
mod fpga_state;
mod gpio;
mod loop_behavior;
mod phase;
mod sampling_config;
mod segment;
mod stm_focus;
mod stm_sampling_config;
mod transition_mode;

pub(crate) use stm_focus::STMFocus;

pub use debug_type::DebugType;
pub use drive::Drive;
pub use emit_intensity::EmitIntensity;
pub use fpga_state::FPGAState;
pub use gpio::*;
pub use loop_behavior::LoopBehavior;
pub use phase::Phase;
pub use sampling_config::SamplingConfig;
pub use segment::Segment;
pub use stm_sampling_config::STMSamplingConfig;
pub use transition_mode::*;

use crate::defined::mm;

pub const FPGA_BASE_CLK_FREQ: u32 = 25600000;
pub const ULTRASOUND_PERIOD: u32 = 512;

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

pub const SILENCER_VALUE_MIN: u16 = 1;
pub const SILENCER_VALUE_MAX: u16 = 0xFFFF;
pub const SILENCER_STEPS_INTENSITY_DEFAULT: u16 = 10;
pub const SILENCER_STEPS_PHASE_DEFAULT: u16 = 40;

pub const SAMPLING_FREQ_DIV_MIN: u32 = 512;
pub const SAMPLING_FREQ_DIV_MAX: u32 = u32::MAX;

pub const MOD_BUF_SIZE_MIN: usize = 2;
pub const MOD_BUF_SIZE_MAX: usize = 32768;

pub const STM_BUF_SIZE_MIN: usize = 2;
pub const FOCI_STM_FOCI_NUM_MAX: usize = 8;
pub const FOCI_STM_BUF_SIZE_MAX: usize = 8192;
pub const GAIN_STM_BUF_SIZE_MAX: usize = 1024;

pub const PWE_BUF_SIZE: usize = 32768;
pub const PULSE_WIDTH_MAX: u16 = ULTRASOUND_PERIOD as u16 / 2;

pub const DRP_ROM_SIZE: usize = 32;
