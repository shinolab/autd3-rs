use std::time::Duration;

use crate::derive::{AUTDInternalError, Geometry};

use super::Datagram;

use derive_more::Deref;

#[derive(Deref, Debug)]
pub struct DatagramWithTimeout<D: Datagram> {
    #[deref]
    datagram: D,
    timeout: Option<Duration>,
}

impl<D: Datagram> Datagram for DatagramWithTimeout<D> {
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        self.datagram.operation_generator(geometry)
    }

    fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    fn parallel_threshold(&self) -> Option<usize> {
        self.datagram.parallel_threshold()
    }
}

pub trait IntoDatagramWithTimeout<D: Datagram> {
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
