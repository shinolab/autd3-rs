use std::time::Duration;

use crate::derive::{AUTDInternalError, Device, Geometry};

use super::Datagram;

pub struct DatagramWithTimeout<'a, D: Datagram<'a>> {
    datagram: D,
    timeout: Duration,
    _phantom: std::marker::PhantomData<&'a D>,
}

impl<'a, D: Datagram<'a>> Datagram<'a> for DatagramWithTimeout<'a, D> {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(
        &'a self,
        geometry: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        self.datagram.operation(geometry)
    }

    fn timeout(&self) -> Option<Duration> {
        Some(self.timeout)
    }
}

pub trait IntoDatagramWithTimeout<'a, D: Datagram<'a>> {
    fn with_timeout(self, timeout: Duration) -> DatagramWithTimeout<'a, D>;
}

impl<'a, D: Datagram<'a>> IntoDatagramWithTimeout<'a, D> for D {
    fn with_timeout(self, timeout: Duration) -> DatagramWithTimeout<'a, D> {
        DatagramWithTimeout {
            datagram: self,
            timeout,
            _phantom: std::marker::PhantomData,
        }
    }
}
