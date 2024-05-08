use std::time::Duration;

use super::Datagram;
use crate::firmware::{
    fpga::{Segment, TransitionMode},
    operation::Operation,
};

/// Datagram with target segment
pub struct DatagramWithSegmentTransition<D: DatagramST> {
    datagram: D,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<D: DatagramST> DatagramWithSegmentTransition<D> {
    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl<D: DatagramST> DatagramWithSegmentTransition<D> {
    pub const fn transition_mode(&self) -> Option<TransitionMode> {
        self.transition_mode
    }
}

impl<D: DatagramST> std::ops::Deref for DatagramWithSegmentTransition<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.datagram
    }
}

impl<D: DatagramST> Datagram for DatagramWithSegmentTransition<D> {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(self) -> (Self::O1, Self::O2) {
        self.datagram
            .operation_with_segment(self.segment, self.transition_mode)
    }

    fn timeout(&self) -> Option<Duration> {
        self.datagram.timeout()
    }
}

impl<D: DatagramST> Datagram for D {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(self) -> (Self::O1, Self::O2) {
        <Self as DatagramST>::operation_with_segment(
            self,
            Segment::S0,
            Some(TransitionMode::Immidiate),
        )
    }

    fn timeout(&self) -> Option<Duration> {
        <Self as DatagramST>::timeout(self)
    }
}

pub trait DatagramST {
    type O1: Operation;
    type O2: Operation;

    fn operation_with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> (Self::O1, Self::O2);

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

pub trait IntoDatagramWithSegmentTransition<D: DatagramST> {
    /// Set segment
    fn with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> DatagramWithSegmentTransition<D>;
}

impl<D: DatagramST> IntoDatagramWithSegmentTransition<D> for D {
    fn with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> DatagramWithSegmentTransition<D> {
        DatagramWithSegmentTransition {
            datagram: self,
            segment,
            transition_mode,
        }
    }
}

impl<D: DatagramST + Clone> Clone for DatagramWithSegmentTransition<D> {
    fn clone(&self) -> Self {
        Self {
            datagram: self.datagram.clone(),
            segment: self.segment,
            transition_mode: self.transition_mode,
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
    impl DatagramST for TestDatagram {
        type O1 = ClearOp;
        type O2 = NullOp;

        fn operation_with_segment(
            self,
            _segment: Segment,
            _transition_mode: Option<TransitionMode>,
        ) -> (Self::O1, Self::O2) {
            (Self::O1::default(), Self::O2::default())
        }
    }

    #[test]
    fn test() {
        let d: DatagramWithSegmentTransition<TestDatagram> =
            TestDatagram { data: 0 }.with_segment(Segment::S0, Some(TransitionMode::SyncIdx));

        assert_eq!(None, d.timeout());
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(TransitionMode::SyncIdx), d.transition_mode());

        let _: (ClearOp, NullOp) = d.operation();
    }

    #[test]
    fn test_derive() {
        let data = 1;
        let d: DatagramWithSegmentTransition<TestDatagram> =
            TestDatagram { data }.with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
        let c = d.clone();
        assert_eq!(d.data, c.data);
    }
}
