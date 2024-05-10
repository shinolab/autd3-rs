use std::time::Duration;

use super::Datagram;

/// Datagram with timeout
pub struct DatagramWithTimeout<D: Datagram> {
    datagram: D,
    timeout: Duration,
}

impl<D: Datagram> Datagram for DatagramWithTimeout<D> {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(self) -> (Self::O1, Self::O2) {
        self.datagram.operation()
    }

    fn timeout(&self) -> Option<Duration> {
        Some(self.timeout)
    }
}

pub trait IntoDatagramWithTimeout<D: Datagram> {
    /// Set timeout.
    /// This takes precedence over the timeout specified in Link.
    fn with_timeout(self, timeout: Duration) -> DatagramWithTimeout<D>;
}

impl<D: Datagram> IntoDatagramWithTimeout<D> for D {
    fn with_timeout(self, timeout: Duration) -> DatagramWithTimeout<D> {
        DatagramWithTimeout {
            datagram: self,
            timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::operation::{ClearOp, NullOp};

    use super::*;

    struct TestDatagram {}
    impl Datagram for TestDatagram {
        type O1 = ClearOp;
        type O2 = NullOp;

        fn operation(self) -> (Self::O1, Self::O2) {
            (Self::O1::default(), Self::O2::default())
        }
    }

    #[test]
    fn test_timeout() {
        let d: DatagramWithTimeout<TestDatagram> =
            TestDatagram {}.with_timeout(Duration::from_millis(100));

        let timeout = <DatagramWithTimeout<TestDatagram> as Datagram>::timeout(&d);
        assert!(timeout.is_some());
        assert_eq!(timeout.unwrap(), Duration::from_millis(100));

        let _: (ClearOp, NullOp) = <DatagramWithTimeout<TestDatagram> as Datagram>::operation(d);
    }
}
