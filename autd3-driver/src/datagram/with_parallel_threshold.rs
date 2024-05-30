use crate::derive::{AUTDInternalError, Geometry};

use super::Datagram;

use derive_more::Deref;

#[derive(Clone, Deref)]
pub struct DatagramWithParallelThreshold<'a, D: Datagram> {
    #[deref]
    datagram: D,
    threshold: usize,
    _phantom: std::marker::PhantomData<&'a D>,
}

impl<'a, D: Datagram> Datagram for DatagramWithParallelThreshold<'a, D> {
    type O1 = D::O1;
    type O2 = D::O2;
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        self.datagram.operation_generator(geometry)
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        self.datagram.timeout()
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(self.threshold)
    }
}

pub trait IntoDatagramWithParallelThreshold<'a, D: Datagram> {
    fn with_paralle_threshold(self, threshold: usize) -> DatagramWithParallelThreshold<'a, D>;
}

impl<'a, D: Datagram> IntoDatagramWithParallelThreshold<'a, D> for D {
    fn with_paralle_threshold(self, threshold: usize) -> DatagramWithParallelThreshold<'a, D> {
        DatagramWithParallelThreshold {
            datagram: self,
            threshold,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        datagram::tests::{NullDatagram, NullOperationGenerator},
        defined::FREQ_40K,
        geometry::tests::create_geometry,
    };

    #[test]
    fn with_parallel_threshold() {
        let geometry = create_geometry(1, 249, FREQ_40K);
        let datagram = NullDatagram {
            timeout: Some(std::time::Duration::from_secs(1)),
            parallel_threshold: None,
        }
        .with_paralle_threshold(100);
        assert_eq!(datagram.timeout(), Some(std::time::Duration::from_secs(1)));
        assert_eq!(datagram.parallel_threshold(), Some(100));
        let _: Result<NullOperationGenerator, _> = datagram.operation_generator(&geometry);
    }
}
