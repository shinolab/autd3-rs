use std::time::Duration;

use super::Datagram;
use crate::{
    derive::{AUTDInternalError, Device, Geometry},
    firmware::{fpga::Segment, operation::Operation},
};

pub struct DatagramWithSegment<'a, D: DatagramS<'a>> {
    datagram: D,
    segment: Segment,
    transition: bool,
    _phantom: std::marker::PhantomData<&'a D>,
}

impl<'a, D: DatagramS<'a>> DatagramWithSegment<'a, D> {
    pub const fn segment(&self) -> Segment {
        self.segment
    }

    pub const fn transition(&self) -> bool {
        self.transition
    }
}

impl<'a, D: DatagramS<'a>> std::ops::Deref for DatagramWithSegment<'a, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.datagram
    }
}

impl<'a, D: DatagramS<'a>> Datagram<'a> for DatagramWithSegment<'a, D> {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(
        &'a self,
        geometry: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        self.datagram
            .operation_with_segment(geometry, self.segment, self.transition)
    }

    fn timeout(&self) -> Option<Duration> {
        self.datagram.timeout()
    }
}

pub trait DatagramS<'a> {
    type O1: Operation + 'a;
    type O2: Operation + 'a;

    fn operation_with_segment(
        &'a self,
        geometry: &'a Geometry,
        segment: Segment,
        transition: bool,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

pub trait IntoDatagramWithSegment<'a, D: DatagramS<'a>> {
    fn with_segment(self, segment: Segment, transition: bool) -> DatagramWithSegment<'a, D>;
}

impl<'a, D: DatagramS<'a>> IntoDatagramWithSegment<'a, D> for D {
    fn with_segment(self, segment: Segment, transition: bool) -> DatagramWithSegment<'a, D> {
        DatagramWithSegment {
            datagram: self,
            segment,
            transition,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, D: DatagramS<'a> + Clone> Clone for DatagramWithSegment<'a, D> {
    fn clone(&self) -> Self {
        Self {
            datagram: self.datagram.clone(),
            segment: self.segment,
            transition: self.transition,
            _phantom: std::marker::PhantomData,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::firmware::operation::{ClearOp, NullOp};

//     use super::*;

//     #[derive(Clone)]
//     struct TestDatagram {
//         pub data: i32,
//     }
//     impl DatagramS for TestDatagram {
//         type O1 = ClearOp;
//         type O2 = NullOp;

//         fn operation_with_segment(
//             self,
//             _segment: Segment,
//             _transition: bool,
//         ) -> (Self::O1, Self::O2) {
//             (Self::O1::default(), Self::O2::default())
//         }
//     }

//     #[test]
//     fn test() {
//         let d: DatagramWithSegment<TestDatagram> =
//             TestDatagram { data: 0 }.with_segment(Segment::S0, true);

//         assert_eq!(None, d.timeout());
//         assert_eq!(Segment::S0, d.segment());
//         assert!(d.transition());

//         let _: (ClearOp, NullOp) = d.operation();
//     }

//     #[test]
//     fn test_derive() {
//         let data = 1;
//         let d: DatagramWithSegment<TestDatagram> =
//             TestDatagram { data }.with_segment(Segment::S0, true);
//         let c = d.clone();
//         assert_eq!(d.data, c.data);
//     }
// }
