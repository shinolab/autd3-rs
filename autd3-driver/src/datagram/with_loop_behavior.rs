use autd3_core::{
    datagram::{
        Datagram, DatagramL, DatagramOption, DeviceFilter, Inspectable, InspectionResult,
        LoopBehavior, Segment, TransitionMode,
    },
    derive::FirmwareLimits,
    geometry::Geometry,
};

use derive_more::Deref;

/// A wrapper of [`DatagramL`] to specify the loop behavior.
///
/// Note that the loop behavior only affects when switching segments.
#[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithLoopBehavior<D: DatagramL> {
    #[deref]
    /// The original [`DatagramL`]
    pub inner: D,
    /// The loop behavior
    pub loop_behavior: LoopBehavior,
    /// The segment to write the data
    pub segment: Segment,
    /// The behavior when switching segments
    pub transition_mode: Option<TransitionMode>,
}

impl<D: DatagramL> WithLoopBehavior<D> {
    /// Create a new [`WithLoopBehavior`].
    #[must_use]
    pub const fn new(
        inner: D,
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self {
        Self {
            inner,
            loop_behavior,
            segment,
            transition_mode,
        }
    }
}

impl<D: DatagramL> Datagram for WithLoopBehavior<D> {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        <D as DatagramL>::operation_generator_with_loop_behavior(
            self.inner,
            geometry,
            filter,
            limits,
            self.segment,
            self.transition_mode,
            self.loop_behavior,
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramL>::option(&self.inner)
    }
}

#[doc(hidden)]
pub trait InspectionResultWithLoopBehavior {
    fn with_loop_behavior(
        self,
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self;
}

impl<D> Inspectable for WithLoopBehavior<D>
where
    D: Inspectable + DatagramL,
    D::Result: InspectionResultWithLoopBehavior,
    <D as DatagramL>::Error: From<<D as Datagram>::Error>,
{
    type Result = D::Result;

    fn inspect(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<InspectionResult<Self::Result>, Self::Error> {
        Ok(self.inner.inspect(geometry, filter, limits)?.modify(|t| {
            t.with_loop_behavior(self.loop_behavior, self.segment, self.transition_mode)
        }))
    }
}
