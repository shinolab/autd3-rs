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
