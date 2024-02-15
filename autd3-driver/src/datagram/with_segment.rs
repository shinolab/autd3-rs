use std::time::Duration;

use super::Datagram;
use crate::cpu::Segment;
use crate::error::AUTDInternalError;
use crate::operation::Operation;

/// Datagram with target segment
pub struct DatagramWithSegment<D: DatagramS> {
    datagram: D,
    pub(crate) segment: Segment,
    pub(crate) update_segment: bool,
}

impl<D: DatagramS> Datagram for DatagramWithSegment<D> {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        self.datagram
            .operation_with_segment(self.segment, self.update_segment)
    }

    fn timeout(&self) -> Option<Duration> {
        self.datagram.timeout()
    }
}

pub trait DatagramS {
    type O1: Operation;
    type O2: Operation;

    fn operation_with_segment(
        self,
        segment: Segment,
        update_segment: bool,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

impl<D: DatagramS> Datagram for D {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        <Self as DatagramS>::operation_with_segment(self, Segment::S0, true)
    }

    fn timeout(&self) -> Option<Duration> {
        <Self as DatagramS>::timeout(self)
    }
}

pub trait IntoDatagramWithSegment<D: DatagramS> {
    /// Set segment
    fn with_segment(self, segment: Segment, update: bool) -> DatagramWithSegment<D>;
}
