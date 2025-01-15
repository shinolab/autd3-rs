use std::time::Duration;

use super::Datagram;
use crate::{
    firmware::fpga::{Segment, TransitionMode},
    geometry::Geometry,
};

use autd3_core::datagram::DatagramS;
use autd3_derive::Builder;
use derive_more::Deref;

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
    type Error = D::Error;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, Self::Error> {
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
