use autd3_core::{
    datagram::{
        Datagram, DatagramOption, DatagramS, DeviceFilter, Inspectable, InspectionResult, Segment,
        TransitionMode,
    },
    derive::FirmwareLimits,
    geometry::Geometry,
};

use derive_more::Deref;

/// A wrapper of [`DatagramS`] to specify the segment to write the data.
#[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithSegment<D: DatagramS> {
    #[deref]
    /// The original [`DatagramS`]
    pub inner: D,
    /// The segment to write the data
    pub segment: Segment,
    /// The behavior when switching segments
    pub transition_mode: Option<TransitionMode>,
}

impl<D: DatagramS> WithSegment<D> {
    /// Create a new [`WithSegment`].
    #[must_use]
    pub const fn new(inner: D, segment: Segment, transition_mode: Option<TransitionMode>) -> Self {
        Self {
            inner,
            segment,
            transition_mode,
        }
    }
}

impl<D: DatagramS> Datagram for WithSegment<D> {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        <D as DatagramS>::operation_generator_with_segment(
            self.inner,
            geometry,
            filter,
            limits,
            self.segment,
            self.transition_mode,
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(&self.inner)
    }
}

#[doc(hidden)]
pub trait InspectionResultWithSegment {
    fn with_segment(self, segment: Segment, transition_mode: Option<TransitionMode>) -> Self;
}

impl<D> Inspectable for WithSegment<D>
where
    D: Inspectable + DatagramS,
    D::Result: InspectionResultWithSegment,
    <D as DatagramS>::Error: From<<D as Datagram>::Error>,
{
    type Result = D::Result;

    fn inspect(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<InspectionResult<Self::Result>, Self::Error> {
        Ok(self
            .inner
            .inspect(geometry, filter, limits)?
            .modify(|t| t.with_segment(self.segment, self.transition_mode)))
    }
}
