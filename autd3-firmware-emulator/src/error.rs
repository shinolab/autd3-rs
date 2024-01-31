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
        assert_eq!("The input data is invalid.", format!("{}", err));
        assert_eq!("InvalidDateTime", format!("{:?}", err));
    }
}
