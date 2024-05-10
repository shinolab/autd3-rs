use crate::{datagram::*, derive::DEFAULT_TIMEOUT, geometry::Device};

/// Datagram for configure force fan
pub struct ForceFan<F: Fn(&Device) -> bool> {
    f: F,
}

impl<F: Fn(&Device) -> bool> ForceFan<F> {
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

impl<F: Fn(&Device) -> bool> Datagram for ForceFan<F> {
    type O1 = crate::firmware::operation::ForceFanOp<F>;
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
    fn f(dev: &Device) -> bool {
        dev.idx() == 0
    }
    // GRCOV_EXCL_STOP

    #[test]
    fn test_timeout() {
        let datagram = ForceFan::new(f);
        let timeout = datagram.timeout();
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_operation() {
        let datagram = ForceFan::new(f);
        let _ = datagram.operation();
    }
}
