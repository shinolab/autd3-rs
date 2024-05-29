use std::time::Duration;

use super::{Datagram, OperationGenerator};
use crate::{
    derive::{AUTDInternalError, Geometry},
    firmware::{fpga::Segment, operation::Operation},
};

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

pub trait DatagramS {
    type O1: Operation;
    type O2: Operation;
    type G: OperationGenerator<O1 = Self::O1, O2 = Self::O2>;

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

impl<D: DatagramS + Clone> Clone for DatagramWithSegment<D> {
    fn clone(&self) -> Self {
        Self {
            datagram: self.datagram.clone(),
            segment: self.segment,
            transition: self.transition,
        }
    }
}
