/*
 * File: error.rs
 * Project: src
 * Created Date: 14/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 14/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use thiserror::Error;

#[derive(Error, Debug)]
#[deprecated(since = "19.1.0", note = "This error is no longer used.")]
pub enum AUTDExtraError {
    #[error("The size of local_trans_pos is wrong.")]
    FPGALocalTransPos,
}
