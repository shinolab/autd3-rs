use std::time::Duration;

use super::{Datagram, OperationGenerator};
use crate::{
    derive::*,
    firmware::fpga::{Segment, TransitionMode},
};

use derive_more::Deref;

#[derive(Builder, Clone, Deref)]

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

    #[tracing::instrument(level = "debug", skip(self, geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!(
            "{} ({:?}, {:?})",
            tynm::type_name::<Self>(),
            self.segment,
            self.transition_mode
        );
        self.datagram.trace(geometry);
    }
    // GRCOV_EXCL_STOP
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

    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        <Self as DatagramST>::trace(self, geometry);
    }
    // GRCOV_EXCL_STOP
}

pub trait DatagramST {
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

    #[tracing::instrument(skip(self, _geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
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

#[cfg(feature = "capi")]
impl<D: DatagramST + Default> Default for DatagramWithSegmentTransition<D> {
    fn default() -> Self {
        Self {
            datagram: D::default(),
            segment: Segment::default(),
            transition_mode: None,
        }
    }
}
