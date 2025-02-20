use derive_more::Display;
use thiserror::Error;

#[derive(Error, Debug, Display, PartialEq, Clone)]
#[display("{}", msg)]
/// An error produced by the link.
pub struct LinkError {
    msg: String,
}

impl LinkError {
    /// Creates a new [`LinkError`].
    pub fn new(msg: impl ToString) -> LinkError {
        LinkError {
            msg: msg.to_string(),
        }
    }
}
