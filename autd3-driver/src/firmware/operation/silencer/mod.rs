mod completion_steps;
mod update_rate;

pub const SILENCER_FLAG_BIT_FIXED_UPDATE_RATE_MODE: u8 = 0;
pub const SILENCER_FLAG_FIXED_UPDATE_RATE_MODE: u8 = 1 << SILENCER_FLAG_BIT_FIXED_UPDATE_RATE_MODE;
pub const SILENCER_FLAG_BIT_PULSE_WIDTH: u8 = 1;
pub const SILENCER_FLAG_PULSE_WIDTH: u8 = 1 << SILENCER_FLAG_BIT_PULSE_WIDTH;
const SILENCER_FLAG_STRICT_MODE: u8 = 1 << 2;

pub use completion_steps::SilencerFixedCompletionStepsOp;
pub use update_rate::SilencerFixedUpdateRateOp;
