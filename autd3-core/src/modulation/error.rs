use core::convert::Infallible;

use alloc::string::{String, ToString};

use crate::firmware::SamplingConfigError;

#[derive(Debug, PartialEq, Clone)]
/// An error occurred during modulation calculation.
pub struct ModulationError {
    msg: String,
}

// GRCOV_EXCL_START
impl core::fmt::Display for ModulationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl core::error::Error for ModulationError {}

impl ModulationError {
    /// Creates a new [`ModulationError`].
    #[must_use]
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

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
