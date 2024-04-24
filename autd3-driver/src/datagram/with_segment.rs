use std::time::Duration;

use super::Datagram;
use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TransitionMode},
        operation::Operation,
    },
};

/// Datagram with target segment
pub struct DatagramWithSegment<D: DatagramS> {
    datagram: D,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<D: DatagramS> DatagramWithSegment<D> {
    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl<D: DatagramS> DatagramWithSegment<D> {
    pub const fn transition_mode(&self) -> Option<TransitionMode> {
        self.transition_mode
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
            .operation_with_segment(self.segment, self.transition_mode)
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
        transition_mode: Option<TransitionMode>,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

impl<D: DatagramS> Datagram for D {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        <Self as DatagramS>::operation_with_segment(
            self,
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        )
    }

    fn timeout(&self) -> Option<Duration> {
        <Self as DatagramS>::timeout(self)
    }
}

pub trait IntoDatagramWithSegment<D: DatagramS> {
    /// Set segment
    fn with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> DatagramWithSegment<D>;
}

impl<D: DatagramS> IntoDatagramWithSegment<D> for D {
    fn with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> DatagramWithSegment<D> {
        DatagramWithSegment {
            datagram: self,
            segment,
            transition_mode,
        }
    }
}

impl<D: DatagramS + Clone> Clone for DatagramWithSegment<D> {
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
    impl DatagramS for TestDatagram {
        type O1 = ClearOp;
        type O2 = NullOp;

        fn operation_with_segment(
            self,
            _segment: Segment,
            _transition_mode: Option<TransitionMode>,
        ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
            Ok((Self::O1::default(), Self::O2::default()))
        }
    }

    #[test]
    fn test() {
        let d: DatagramWithSegment<TestDatagram> =
            TestDatagram { data: 0 }.with_segment(Segment::S0, Some(TransitionMode::SyncIdx));

        assert_eq!(None, d.timeout());
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(TransitionMode::SyncIdx), d.transition_mode());

        let _: (ClearOp, NullOp) = d.operation().unwrap();
    }

    #[test]
    fn test_derive() {
        let data = 1;
        let d: DatagramWithSegment<TestDatagram> =
            TestDatagram { data }.with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
        let c = d.clone();
        assert_eq!(d.data, c.data);
    }
}
