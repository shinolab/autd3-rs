use std::time::Duration;

use super::{Datagram, OperationGenerator};
use crate::{
    defined::DEFAULT_TIMEOUT,
    error::AUTDDriverError,
    firmware::fpga::{Segment, TransitionMode},
    geometry::Geometry,
};

use autd3_derive::Builder;
use derive_more::Deref;

/// [`DatagramS`] represents a [`Datagram`] that can specify [`Segment`] to write the data.
pub trait DatagramS: std::fmt::Debug {
    #[doc(hidden)]
    type G: OperationGenerator;

    #[doc(hidden)]
    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDDriverError>;

    /// Returns the timeout duration.
    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    /// Returns the parallel threshold.
    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}

/// A wrapper to set [`Segment`] of [`DatagramS`].
#[derive(Builder, Clone, Deref, Debug)]
pub struct DatagramWithSegment<D: DatagramS> {
    #[deref]
    datagram: D,
    #[get]
    /// Segment to write the data.
    segment: Segment,
    #[get]
    /// Transition mode. If `None`, the data is written to the segment, but the transition does not occur.     
    /// See [`TransitionMode`] for details.
    transition_mode: Option<TransitionMode>,
}

impl<D: DatagramS> Datagram for DatagramWithSegment<D> {
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDDriverError> {
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

impl<D: DatagramS> Datagram for D {
    type G = D::G;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDDriverError> {
        self.operation_generator_with_segment(
            geometry,
            Segment::S0,
            Some(TransitionMode::Immediate),
        )
    }

    fn timeout(&self) -> Option<Duration> {
        <Self as DatagramS>::timeout(self)
    }

    fn parallel_threshold(&self) -> Option<usize> {
        <Self as DatagramS>::parallel_threshold(self)
    }
}

/// A trait to convert [`DatagramS`] to [`DatagramWithSegment`].
pub trait IntoDatagramWithSegment<D: DatagramS> {
    /// Convert [`DatagramS`] to [`DatagramWithSegment`].
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
