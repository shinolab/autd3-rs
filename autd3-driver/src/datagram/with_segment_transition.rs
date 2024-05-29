use std::time::Duration;

use super::{Datagram, OperationGenerator};
use crate::{
    derive::{AUTDInternalError, Geometry},
    firmware::{
        fpga::{Segment, TransitionMode},
        operation::Operation,
    },
};

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
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        self.datagram
            .operation_generator_with_segment(geometry, self.segment, self.transition_mode)
    }

    fn timeout(&self) -> Option<Duration> {
        self.datagram.timeout()
    }

    fn parallel_threshold(&self) -> Option<usize> {
        self.datagram.parallel_threshold()
    }
}

impl<D: DatagramST> Datagram for D {
    type O1 = D::O1;
    type O2 = D::O2;
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        self.operation_generator_with_segment(
            geometry,
            Segment::S0,
            Some(TransitionMode::Immediate),
        )
    }

    fn timeout(&self) -> Option<Duration> {
        <Self as DatagramST>::timeout(self)
    }

    fn parallel_threshold(&self) -> Option<usize> {
        <Self as DatagramST>::parallel_threshold(self)
    }
}

pub trait DatagramST {
    type O1: Operation;
    type O2: Operation;
    type G: OperationGenerator<O1 = Self::O1, O2 = Self::O2>;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }

    fn parallel_threshold(&self) -> Option<usize> {
        None
    }
}

pub trait IntoDatagramWithSegmentTransition<D: DatagramST> {
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
