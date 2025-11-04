#[derive(Debug, PartialEq, Clone)]
/// An error occurred during gain calculation.
pub struct GainError {
    msg: String,
}

impl core::fmt::Display for GainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl core::error::Error for GainError {}

impl GainError {
    /// Creates a new [`GainError`].
    #[must_use]
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_error_display() {
        let err = GainError::new("Test error");
        assert_eq!(format!("{}", err), "Test error");
    }
}
