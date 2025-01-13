use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum LinkError {
    #[error("Failed to open the link")]
    Open,
    #[error("Failed to send the data")]
    Send,
    #[error("Failed to receive the data")]
    Receive,
    #[error("Failed to close the link")]
    Close,
}
