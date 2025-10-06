use alloc::string::{String, ToString};

#[derive(Debug, PartialEq, Clone)]
/// An error occurred during gain calculation.
pub struct GainError {
    msg: String,
}

// GRCOV_EXCL_START
impl core::fmt::Display for GainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl core::error::Error for GainError {}
// GRCOV_EXCL_STOP

impl GainError {
    /// Creates a new [`GainError`].
    #[must_use]
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}
