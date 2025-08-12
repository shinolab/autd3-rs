use autd3_core::{
    datagram::{
        Datagram, DatagramOption, DatagramS, DeviceMask, Inspectable, InspectionResult,
        internal::HasSegment,
    },
    environment::Environment,
    firmware::{FirmwareLimits, Segment, transition_mode::TransitionMode},
    geometry::Geometry,
};

use derive_more::Deref;

/// A wrapper of [`DatagramS`] to specify the segment to write the data.
#[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithSegment<T: TransitionMode, D> {
    #[deref]
    /// The original [`DatagramS`]
    pub inner: D,
    /// The segment to write the data
    pub segment: Segment,
    /// Whether to transition the segment
    pub transition_mode: T,
}

impl<'a, T, D> WithSegment<T, D>
where
    T: TransitionMode,
    D: DatagramS<'a> + HasSegment<T>,
{
    /// Create a new [`WithSegment`].
    #[must_use]
    pub const fn new(inner: D, segment: Segment, transition_mode: T) -> Self {
        Self {
            inner,
            segment,
            transition_mode,
        }
    }
}

impl<'a, T, D> Datagram<'a> for WithSegment<T, D>
where
    T: TransitionMode,
    D: DatagramS<'a> + HasSegment<T>,
{
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        <D as DatagramS>::operation_generator_with_segment(
            self.inner,
            geometry,
            env,
            filter,
            limits,
            self.segment,
            self.transition_mode.params(),
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(&self.inner)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithSegmentInspectionResult<I, T: TransitionMode> {
    pub inner: I,
    pub segment: Segment,
    pub transition_mode: T,
}

impl<'a, T, D> Inspectable<'a> for WithSegment<T, D>
where
    T: TransitionMode,
    D: Inspectable<'a> + DatagramS<'a> + HasSegment<T>,
    <D as DatagramS<'a>>::Error: From<<D as Datagram<'a>>::Error>,
{
    type Result = WithSegmentInspectionResult<D::Result, T>;

    fn inspect(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        limits: &FirmwareLimits,
    ) -> Result<InspectionResult<Self::Result>, Self::Error> {
        Ok(InspectionResult {
            result: self
                .inner
                .inspect(geometry, env, filter, limits)?
                .result
                .into_iter()
                .map(|r| {
                    r.map(|r| WithSegmentInspectionResult {
                        inner: r,
                        segment: self.segment,
                        transition_mode: self.transition_mode,
                    })
                })
                .collect(),
        })
    }
}
