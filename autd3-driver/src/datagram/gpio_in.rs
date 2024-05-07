use crate::{datagram::*, derive::DEFAULT_TIMEOUT, geometry::Device};

/// Datagram for configure force fan
pub struct EmulateGPIOIn<F: Fn(&Device) -> [bool; 4]> {
    f: F,
}

impl<F: Fn(&Device) -> [bool; 4]> EmulateGPIOIn<F> {
    /// constructor
    pub const fn new(f: F) -> Self {
        Self { f }
    }

    /// Get the function
    // GRCOV_EXCL_START
    pub fn f(&self) -> &F {
        &self.f
    }
    // GRCOV_EXCL_STOP
}

impl<F: Fn(&Device) -> [bool; 4]> Datagram for EmulateGPIOIn<F> {
    type O1 = crate::firmware::operation::EmulateGPIOInOp<F>;
    type O2 = crate::firmware::operation::NullOp;

    fn operation(self) -> (Self::O1, Self::O2) {
        (Self::O1::new(self.f), Self::O2::default())
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GRCOV_EXCL_START
    fn f(_: &Device) -> [bool; 4] {
        [true; 4]
    }
    // GRCOV_EXCL_STOP

    #[test]
    fn test_timeout() {
        let datagram = EmulateGPIOIn::new(f);
        let timeout = datagram.timeout();
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_operation() {
        let datagram = EmulateGPIOIn::new(f);
        let _ = datagram.operation();
    }
}
