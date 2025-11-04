use crate::firmware::SamplingConfigError;

#[derive(Debug, PartialEq, Clone)]
/// An error occurred during modulation calculation.
pub struct ModulationError {
    msg: String,
}

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

impl From<SamplingConfigError> for ModulationError {
    fn from(e: SamplingConfigError) -> Self {
        Self::new(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let err = ModulationError::new("test error");
        assert_eq!(format!("{}", err), "test error");
    }

    #[test]
    fn from_sampling_config_error() {
        let sce = SamplingConfigError::FreqInvalid(1 * crate::common::Hz);
        let me: ModulationError = sce.into();
        assert_eq!(me.msg, sce.to_string());
    }
}
