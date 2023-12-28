/*
 * File: mod.rs
 * Project: silencer
 * Created Date: 27/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 27/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

mod completion_steps;
mod update_rate;

const SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS_BIT: u16 = 0;
const SILENCER_CTL_FLAG_STRICT_MODE_BIT: u16 = 8;

const SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS: u16 =
    1 << SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS_BIT;
const SILENCER_CTL_FLAG_STRICT_MODE: u16 = 1 << SILENCER_CTL_FLAG_STRICT_MODE_BIT;

pub use completion_steps::ConfigSilencerFixedCompletionStepsOp;
pub use update_rate::ConfigSilencerFixedUpdateRateOp;
