use crate::datagram::*;

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

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Default::default(), Default::default()))
    }

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
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
        let r = <Synchronize as Datagram>::operation(stop);
        assert!(r.is_ok());
        let _: (
            crate::firmware::operation::SyncOp,
            crate::firmware::operation::NullOp,
        ) = r.unwrap();
    }
}
