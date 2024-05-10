use crate::{datagram::*, defined::DEFAULT_TIMEOUT};

/// Datagram for clear all data in devices
#[derive(Default)]
pub struct Clear {}

impl Clear {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Datagram for Clear {
    type O1 = crate::firmware::operation::ClearOp;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(self) -> (Self::O1, Self::O2) {
        (Self::O1::default(), Self::O2::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::operation::{ClearOp, NullOp};

    use super::*;

    #[test]
    fn test_timeout() {
        let clear = Clear::new();
        let timeout = <Clear as Datagram>::timeout(&clear);
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_operation() {
        let clear = Clear::default();
        let _: (ClearOp, NullOp) = clear.operation();
    }
}
