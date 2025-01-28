use autd3_core::datagram::{Datagram, DatagramOption, DatagramS, Segment, TransitionMode};

use derive_more::Deref;
use derive_new::new;

/// A wrapper of [`DatagramS`] to specify the segment to write the data.
#[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash, new)]
pub struct WithSegment<D: DatagramS> {
    #[deref]
    /// The original [`DatagramS`]
    pub inner: D,
    /// The segment to write the data
    pub segment: Segment,
    /// The behavior when switching segments
    pub transition_mode: Option<TransitionMode>,
}

impl<D: DatagramS> Datagram for WithSegment<D> {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &autd3_core::derive::Geometry,
        parallel: bool,
    ) -> Result<Self::G, Self::Error> {
        <D as DatagramS>::operation_generator_with_segment(
            self.inner,
            geometry,
            parallel,
            self.segment,
            self.transition_mode,
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(&self.inner)
    }
}
