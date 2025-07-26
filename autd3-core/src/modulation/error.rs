use std::convert::Infallible;

use derive_more::Display;
use thiserror::Error;

use crate::firmware::SamplingConfigError;

#[derive(Error, Debug, Display, PartialEq, Clone)]
#[display("{}", msg)]
/// An error occurred during modulation calculation.
pub struct ModulationError {
    msg: String,
}

impl ModulationError {
    /// Creates a new [`ModulationError`].
    #[must_use]
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

// GRCOV_EXCL_START
impl From<Infallible> for ModulationError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<SamplingConfigError> for ModulationError {
    fn from(e: SamplingConfigError) -> Self {
        Self::new(e)
    }
}
// GRCOV_EXCL_STOP
