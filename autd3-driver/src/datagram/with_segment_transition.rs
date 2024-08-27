use std::time::Duration;

use super::{Datagram, OperationGenerator};
use crate::{
    derive::*,
    firmware::fpga::{Segment, TransitionMode},
};

use derive_more::Deref;

#[derive(Builder, Clone, Deref, Debug)]

pub struct DatagramWithSegmentTransition<D: DatagramST> {
    #[deref]
    datagram: D,
    #[get]
    segment: Segment,
    #[get]
    transition_mode: Option<TransitionMode>,
}

impl<D: DatagramST> Datagram for DatagramWithSegmentTransition<D> {
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

pub trait DatagramST: std::fmt::Debug {
    type G: OperationGenerator;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError>;

    fn timeout(&self) -> Option<Duration>;

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
