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

impl From<std::io::Error> for LinkError {
    fn from(err: std::io::Error) -> Self {
        LinkError::new(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed() {
        let err = LinkError::closed();
        assert_eq!("Link is closed", err.to_string());
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::other("io error");
        let link_err: LinkError = io_err.into();
        assert_eq!("io error", link_err.to_string());
    }
}
