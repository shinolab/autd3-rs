#[derive(Debug, PartialEq, Clone)]
/// An error produced by the link.
pub struct LinkError {
    msg: String,
}

impl core::fmt::Display for LinkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl core::error::Error for LinkError {}

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

// GRCOV_EXCL_START
impl From<std::io::Error> for LinkError {
    fn from(err: std::io::Error) -> Self {
        LinkError::new(err.to_string())
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_error_closed() {
        let err = LinkError::closed();
        assert_eq!("Link is closed", err.to_string());
    }
}
