use crate::{datagram::*, defined::DEFAULT_TIMEOUT};

/// Datagram to synchronize devices
#[derive(Default)]
pub struct Synchronize {}

impl Synchronize {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Datagram for Synchronize {
    type O1 = crate::firmware::operation::SyncOp;
    type O2 = crate::firmware::operation::NullOp;

    fn operation(self) -> (Self::O1, Self::O2) {
        (Default::default(), Default::default())
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout() {
        let stop = Synchronize::new();
        let timeout = <Synchronize as Datagram>::timeout(&stop);
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_operation() {
        let stop = Synchronize::default();
        let _: (
            crate::firmware::operation::SyncOp,
            crate::firmware::operation::NullOp,
        ) = <Synchronize as Datagram>::operation(stop);
    }
}
