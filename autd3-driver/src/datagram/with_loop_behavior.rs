use autd3_core::{
    datagram::{
        Datagram, DatagramL, DatagramOption, DeviceFilter, Inspectable, InspectionResult,
        LoopBehavior, Segment, TransitionMode,
    },
    derive::FirmwareLimits,
    environment::Environment,
    geometry::Geometry,
};

use derive_more::Deref;

/// A wrapper of [`DatagramL`] to specify the loop behavior.
///
/// Note that the loop behavior only affects when switching segments.
#[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithLoopBehavior<D> {
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

impl<'geo, 'dev, 'tr, D: DatagramL<'geo, 'dev, 'tr>> WithLoopBehavior<D> {
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

impl<'geo, 'dev, 'tr, D: DatagramL<'geo, 'dev, 'tr>> Datagram<'geo, 'dev, 'tr>
    for WithLoopBehavior<D>
{
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &'geo Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        <D as DatagramL>::operation_generator_with_loop_behavior(
            self.inner,
            geometry,
            env,
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

impl<'geo, 'dev, 'tr, D> Inspectable<'geo, 'dev, 'tr> for WithLoopBehavior<D>
where
    D: Inspectable<'geo, 'dev, 'tr> + DatagramL<'geo, 'dev, 'tr>,
    D::Result: InspectionResultWithLoopBehavior,
    <D as DatagramL<'geo, 'dev, 'tr>>::Error: From<<D as Datagram<'geo, 'dev, 'tr>>::Error>,
{
    type Result = D::Result;

    fn inspect(
        self,
        geometry: &'geo Geometry,
        env: &autd3_core::environment::Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<InspectionResult<Self::Result>, Self::Error> {
        Ok(self
            .inner
            .inspect(geometry, env, filter, limits)?
            .modify(|t| {
                t.with_loop_behavior(self.loop_behavior, self.segment, self.transition_mode)
            }))
    }
}
