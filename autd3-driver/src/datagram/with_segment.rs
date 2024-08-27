use std::time::Duration;

use super::{Datagram, OperationGenerator};
use crate::{derive::*, firmware::fpga::Segment};

use derive_more::Deref;

#[derive(Builder, Clone, Deref, Debug)]
pub struct DatagramWithSegment<D: DatagramS> {
    #[deref]
    datagram: D,
    #[get]
    segment: Segment,
    #[get]
    transition: bool,
}

impl<D: DatagramS> Datagram for DatagramWithSegment<D> {
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        self.datagram
            .operation_generator_with_segment(geometry, self.segment, self.transition)
    }

    fn timeout(&self) -> Option<Duration> {
        self.datagram.timeout()
    }

    fn parallel_threshold(&self) -> Option<usize> {
        self.datagram.parallel_threshold()
    }
}

pub trait DatagramS: std::fmt::Debug {
    type G: OperationGenerator;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition: bool,
    ) -> Result<Self::G, AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }

    fn parallel_threshold(&self) -> Option<usize> {
        None
    }
}

pub trait IntoDatagramWithSegment<D: DatagramS> {
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
