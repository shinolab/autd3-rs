mod completion_steps;
mod update_rate;

const SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS_BIT: u16 = 0;
const SILENCER_CTL_FLAG_STRICT_MODE_BIT: u16 = 8;

const SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS: u16 =
    1 << SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS_BIT;
const SILENCER_CTL_FLAG_STRICT_MODE: u16 = 1 << SILENCER_CTL_FLAG_STRICT_MODE_BIT;

pub use completion_steps::ConfigSilencerFixedCompletionStepsOp;
pub use update_rate::ConfigSilencerFixedUpdateRateOp;
