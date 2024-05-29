use crate::derive::{AUTDInternalError, Geometry};

use super::Datagram;

pub struct DatagramWithParallelThreshold<'a, D: Datagram<'a>> {
    datagram: D,
    threshold: usize,
    _phantom: std::marker::PhantomData<&'a D>,
}

impl<'a, D: Datagram<'a>> Datagram<'a> for DatagramWithParallelThreshold<'a, D> {
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

pub trait IntoDatagramWithParallelThreshold<'a, D: Datagram<'a>> {
    fn with_paralle_threshold(self, threshold: usize) -> DatagramWithParallelThreshold<'a, D>;
}

impl<'a, D: Datagram<'a>> IntoDatagramWithParallelThreshold<'a, D> for D {
    fn with_paralle_threshold(self, threshold: usize) -> DatagramWithParallelThreshold<'a, D> {
        DatagramWithParallelThreshold {
            datagram: self,
            threshold,
            _phantom: std::marker::PhantomData,
        }
    }
}
