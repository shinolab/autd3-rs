use crate::derive::{AUTDInternalError, Geometry};

use super::Datagram;

use derive_more::Deref;

#[derive(Clone, Deref, Debug)]
pub struct DatagramWithParallelThreshold<D: Datagram> {
    #[deref]
    datagram: D,
    threshold: Option<usize>,
}

impl<D: Datagram> Datagram for DatagramWithParallelThreshold<D> {
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        self.datagram.operation_generator(geometry)
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        self.datagram.timeout()
    }

    fn parallel_threshold(&self) -> Option<usize> {
        self.threshold
    }
}

pub trait IntoDatagramWithParallelThreshold<D: Datagram> {
    fn with_parallel_threshold(self, threshold: Option<usize>) -> DatagramWithParallelThreshold<D>;
}

impl<D: Datagram> IntoDatagramWithParallelThreshold<D> for D {
    fn with_parallel_threshold(self, threshold: Option<usize>) -> DatagramWithParallelThreshold<D> {
        DatagramWithParallelThreshold {
            datagram: self,
            threshold,
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
    #[cfg_attr(miri, ignore)]
    fn with_parallel_threshold() {
        let geometry = create_geometry(1, 249);
        let datagram = NullDatagram {
            timeout: Some(std::time::Duration::from_secs(1)),
            parallel_threshold: None,
        }
        .with_parallel_threshold(Some(100));
        assert_eq!(datagram.timeout(), Some(std::time::Duration::from_secs(1)));
        assert_eq!(datagram.parallel_threshold(), Some(100));
        let _: Result<NullOperationGenerator, _> = datagram.operation_generator(&geometry);
    }
}
