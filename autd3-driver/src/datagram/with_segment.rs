use std::time::Duration;

use super::Datagram;
use crate::firmware::{fpga::Segment, operation::Operation};

/// Datagram with target segment
pub struct DatagramWithSegment<D: DatagramS> {
    datagram: D,
    segment: Segment,
    transition: bool,
}

impl<D: DatagramS> DatagramWithSegment<D> {
    pub const fn segment(&self) -> Segment {
        self.segment
    }

    pub const fn transition(&self) -> bool {
        self.transition
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

    fn operation(self) -> (Self::O1, Self::O2) {
        self.datagram
            .operation_with_segment(self.segment, self.transition)
    }

    fn timeout(&self) -> Option<Duration> {
        self.datagram.timeout()
    }
}

pub trait DatagramS {
    type O1: Operation;
    type O2: Operation;

    fn operation_with_segment(self, segment: Segment, transition: bool) -> (Self::O1, Self::O2);

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

pub trait IntoDatagramWithSegment<D: DatagramS> {
    /// Set segment
    fn with_segment(self, segment: Segment, transition: bool) -> DatagramWithSegment<D>;
}

impl<D: DatagramS> IntoDatagramWithSegment<D> for D {
    fn with_segment(self, segment: Segment, transition: bool) -> DatagramWithSegment<D> {
        DatagramWithSegment {
            datagram: self,
            segment,
            transition,
        }
    }
}

impl<D: DatagramS + Clone> Clone for DatagramWithSegment<D> {
    fn clone(&self) -> Self {
        Self {
            datagram: self.datagram.clone(),
            segment: self.segment,
            transition: self.transition,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::operation::{ClearOp, NullOp};

    use super::*;

    #[derive(Clone)]
    struct TestDatagram {
        pub data: i32,
    }
    impl DatagramS for TestDatagram {
        type O1 = ClearOp;
        type O2 = NullOp;

        fn operation_with_segment(
            self,
            _segment: Segment,
            _transition: bool,
        ) -> (Self::O1, Self::O2) {
            (Self::O1::default(), Self::O2::default())
        }
    }

    #[test]
    fn test() {
        let d: DatagramWithSegment<TestDatagram> =
            TestDatagram { data: 0 }.with_segment(Segment::S0, true);

        assert_eq!(None, d.timeout());
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(true, d.transition());

        let _: (ClearOp, NullOp) = d.operation();
    }

    #[test]
    fn test_derive() {
        let data = 1;
        let d: DatagramWithSegment<TestDatagram> =
            TestDatagram { data }.with_segment(Segment::S0, true);
        let c = d.clone();
        assert_eq!(d.data, c.data);
    }
}
