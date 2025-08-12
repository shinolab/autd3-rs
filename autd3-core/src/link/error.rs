use alloc::string::{String, ToString};
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
    #[must_use]
    pub fn new(msg: impl ToString) -> LinkError {
        LinkError {
            msg: msg.to_string(),
        }
    }

    /// Creates a new [`LinkError`] with a message indicating that the link is closed.
    pub fn closed() -> LinkError {
        LinkError::new("Link is closed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_error_closed() {
        let err = LinkError::closed();
        assert_eq!("Link is closed", err.to_string());
    }
}
