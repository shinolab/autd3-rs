/*
 * File: error.rs
 * Project: src
 * Created Date: 14/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 28/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AUTDFirmwareEmulatorError {
    #[error("The input data is invalid.")]
    InvalidDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn freq_div_out_of_range() {
        let err = AUTDFirmwareEmulatorError::InvalidDateTime;
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "The input data is invalid.");
        assert_eq!(format!("{:?}", err), "InvalidDateTime");
    }
}
