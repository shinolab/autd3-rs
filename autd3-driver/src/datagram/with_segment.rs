use std::time::Duration;

use super::{Datagram, OperationGenerator};
use crate::{
    derive::*,
    firmware::{fpga::Segment, operation::Operation},
};

use derive_more::Deref;

#[derive(Builder, Clone, Deref)]
pub struct DatagramWithSegment<D: DatagramS> {
    #[deref]
    datagram: D,
    #[get]
    segment: Segment,
    #[get]
    transition: bool,
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

    #[tracing::instrument(level = "debug", skip(self, geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::info!(
            "{} ({:?}, {:?})",
            tynm::type_name::<D>(),
            self.segment,
            self.transition
        );
        self.datagram.trace(geometry);
    }
    // GRCOV_EXCL_STOP
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

    fn trace(&self, geometry: &Geometry);
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

#[cfg(feature = "capi")]
impl<D: DatagramS + Default> Default for DatagramWithSegment<D> {
    fn default() -> Self {
        Self {
            datagram: D::default(),
            segment: Segment::default(),
            transition: false,
        }
    }
}
