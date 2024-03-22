use crate::{datagram::*, geometry::Device};

/// Datagram for configure force fan
pub struct ConfigureForceFan<F: Fn(&Device) -> bool> {
    f: F,
}

impl<F: Fn(&Device) -> bool> ConfigureForceFan<F> {
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

impl<F: Fn(&Device) -> bool> Datagram for ConfigureForceFan<F> {
    type O1 = crate::operation::ConfigureForceFanOp<F>;
    type O2 = crate::operation::NullOp;

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.f), Self::O2::default()))
    }

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
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
        let datagram = ConfigureForceFan::new(f);
        let timeout = datagram.timeout();
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_operation() {
        let datagram = ConfigureForceFan::new(f);
        let r = datagram.operation();
        assert!(r.is_ok());
        let _ = r.unwrap();
    }
}
