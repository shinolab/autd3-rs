use crate::{datagram::*, derive::DEFAULT_TIMEOUT};

/// Datagram for configure FPGA clock
#[derive(Default)]
pub struct ConfigureFPGAClock {}

impl ConfigureFPGAClock {
    /// constructor
    pub const fn new() -> Self {
        Self {}
    }
}

impl Datagram for ConfigureFPGAClock {
    type O1 = crate::firmware::operation::ConfigureClockOp;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(self) -> (Self::O1, Self::O2) {
        (Self::O1::new(), Self::O2::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::operation::{ConfigureClockOp, NullOp};

    use super::*;

    #[test]
    fn test_timeout() {
        let d = ConfigureFPGAClock::new();
        let timeout = d.timeout();
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_operation() {
        let d = ConfigureFPGAClock::new();
        let _: (ConfigureClockOp, NullOp) = d.operation();
    }
}
