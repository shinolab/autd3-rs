use crate::defined::{float, METER};

mod drive;
mod fpga_state;
mod stm_focus;

pub use drive::FPGADrive;
pub use fpga_state::FPGAState;
pub use stm_focus::STMFocus;

/// FPGA clock frequency
pub const FPGA_CLK_FREQ: usize = 20480000;

pub const FOCUS_STM_FIXED_NUM_UNIT: float = 0.025e-3 * METER;
pub const FOCUS_STM_FIXED_NUM_WIDTH: usize = 18;
pub const FOCUS_STM_FIXED_NUM_UPPER: i32 = (1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) - 1;
pub const FOCUS_STM_FIXED_NUM_LOWER: i32 = -(1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1));

pub const SILENCER_VALUE_MIN: u16 = 1;
pub const SILENCER_VALUE_MAX: u16 = 0xFFFF;
pub const SILENCER_STEPS_INTENSITY_DEFAULT: u16 = 10;
pub const SILENCER_STEPS_PHASE_DEFAULT: u16 = 40;

pub const SAMPLING_FREQ_DIV_MIN: u32 = 512;
pub const SAMPLING_FREQ_DIV_MAX: u32 = u32::MAX;

pub const MOD_BUF_SIZE_MIN: usize = 2;
pub const MOD_BUF_SIZE_MAX: usize = 65536;

pub const STM_BUF_SIZE_MIN: usize = 2;
pub const FOCUS_STM_BUF_SIZE_MAX: usize = 65536;
pub const GAIN_STM_BUF_SIZE_MAX: usize = 2048;
