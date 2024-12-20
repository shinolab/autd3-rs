use std::time::Duration;

use crate::{error::AUTDDriverError, geometry::Geometry};

use super::Datagram;

use derive_more::Deref;

/// A wrapper to overwrite timeout of [`Datagram`].
#[derive(Deref, Debug)]
pub struct DatagramWithTimeout<D: Datagram> {
    #[deref]
    datagram: D,
    timeout: Option<Duration>,
}

impl<D: Datagram> Datagram for DatagramWithTimeout<D> {
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDDriverError> {
        self.datagram.operation_generator(geometry)
    }

    fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    fn parallel_threshold(&self) -> Option<usize> {
        self.datagram.parallel_threshold()
    }
}

/// A trait to convert [`Datagram`] to [`DatagramWithTimeout`].
pub trait IntoDatagramWithTimeout<D: Datagram> {
    /// Convert [`Datagram`] to [`DatagramWithTimeout`].
    fn with_timeout(self, timeout: Option<Duration>) -> DatagramWithTimeout<D>;
}

impl<D: Datagram> IntoDatagramWithTimeout<D> for D {
    fn with_timeout(self, timeout: Option<Duration>) -> DatagramWithTimeout<D> {
        DatagramWithTimeout {
            datagram: self,
            timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        datagram::tests::{NullDatagram, NullOperationGenerator},
        geometry::tests::create_geometry,
    };

    #[test]
    fn with_timeout() {
        let geometry = create_geometry(1, 249);
        let datagram = NullDatagram {
            timeout: None,
            parallel_threshold: Some(100),
        }
        .with_timeout(Some(std::time::Duration::from_secs(1)));
        assert_eq!(datagram.timeout(), Some(std::time::Duration::from_secs(1)));
        assert_eq!(datagram.parallel_threshold(), Some(100));
        let _: Result<NullOperationGenerator, _> = datagram.operation_generator(&geometry);
    }
}
