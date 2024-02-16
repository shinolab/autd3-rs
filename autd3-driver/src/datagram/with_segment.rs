use std::time::Duration;

use super::Datagram;
use crate::common::Segment;
use crate::error::AUTDInternalError;
use crate::operation::Operation;

/// Datagram with target segment
pub struct DatagramWithSegment<D: DatagramS> {
    datagram: D,
    segment: Segment,
    update_segment: bool,
}

impl<D: DatagramS> DatagramWithSegment<D> {
    pub const fn segment(&self) -> Segment {
        self.segment
    }

    pub const fn update_segment(&self) -> bool {
        self.update_segment
    }
}

impl<D: DatagramS> std::ops::Deref for DatagramWithSegment<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.datagram
    }
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
    fn with_segment(self, segment: Segment, update_segment: bool) -> DatagramWithSegment<D>;
}

impl<D: DatagramS> IntoDatagramWithSegment<D> for D {
    fn with_segment(self, segment: Segment, update_segment: bool) -> DatagramWithSegment<D> {
        DatagramWithSegment {
            datagram: self,
            segment,
            update_segment,
        }
    }
}

impl<D: DatagramS + Clone> Clone for DatagramWithSegment<D> {
    fn clone(&self) -> Self {
        Self {
            datagram: self.datagram.clone(),
            segment: self.segment,
            update_segment: self.update_segment,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operation::{ClearOp, NullOp};

    use super::*;

    struct TestDatagram {}
    impl DatagramS for TestDatagram {
        type O1 = ClearOp;
        type O2 = NullOp;

        fn operation_with_segment(
            self,
            _segment: Segment,
            _update_segment: bool,
        ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
            Ok((Self::O1::default(), Self::O2::default()))
        }
    }

    #[test]
    fn test_datagram_with_segment() {
        let d: DatagramWithSegment<TestDatagram> = TestDatagram {}.with_segment(Segment::S0, true);

        let timeout = <DatagramWithSegment<TestDatagram> as Datagram>::timeout(&d);
        assert!(timeout.is_none());

        let _: (ClearOp, NullOp) =
            <DatagramWithSegment<TestDatagram> as Datagram>::operation(d).unwrap();
    }
}
