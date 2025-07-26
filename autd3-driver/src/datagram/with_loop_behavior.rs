use std::num::NonZeroU16;

use autd3_core::{
    datagram::{
        Datagram, DatagramL, DatagramOption, DeviceFilter, Inspectable, InspectionResult,
        internal::HasFiniteLoop,
    },
    environment::Environment,
    firmware::{FirmwareLimits, Segment, transition_mode::TransitionMode},
    geometry::Geometry,
};

use derive_more::Deref;

/// A wrapper of [`DatagramL`] to specify the loop behavior.
///
/// Note that the loop behavior only affects when switching segments.
#[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithFiniteLoop<T: TransitionMode, D> {
    #[deref]
    /// The original [`DatagramL`]
    pub inner: D,
    /// The loop count
    pub loop_count: NonZeroU16,
    /// The segment to write the data
    pub segment: Segment,
    /// The behavior when switching segments
    pub transition_mode: T,
}

impl<'a, T, D> WithFiniteLoop<T, D>
where
    T: TransitionMode,
    D: DatagramL<'a> + HasFiniteLoop<T>,
{
    /// Create a new [`WithFiniteLoop`].
    #[must_use]
    pub const fn new(
        inner: D,
        loop_count: NonZeroU16,
        segment: Segment,
        transition_mode: T,
    ) -> Self {
        Self {
            inner,
            loop_count,
            segment,
            transition_mode,
        }
    }
}

impl<'a, T, D> Datagram<'a> for WithFiniteLoop<T, D>
where
    T: TransitionMode,
    D: DatagramL<'a> + HasFiniteLoop<T>,
{
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        <D as DatagramL>::operation_generator_with_finite_loop(
            self.inner,
            geometry,
            env,
            filter,
            limits,
            self.segment,
            self.transition_mode.params(),
            self.loop_count.get() - 1,
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramL>::option(&self.inner)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithFiniteLoopInspectionResult<I, T> {
    pub inner: I,
    pub segment: Segment,
    pub transition_mode: T,
    pub loop_count: NonZeroU16,
}

impl<'a, T, D> Inspectable<'a> for WithFiniteLoop<T, D>
where
    T: TransitionMode,
    D: Inspectable<'a> + DatagramL<'a> + HasFiniteLoop<T>,
    <D as DatagramL<'a>>::Error: From<<D as Datagram<'a>>::Error>,
{
    type Result = WithFiniteLoopInspectionResult<D::Result, T>;

    fn inspect(
        self,
        geometry: &'a Geometry,
        env: &autd3_core::environment::Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<InspectionResult<Self::Result>, Self::Error> {
        Ok(InspectionResult {
            result: self
                .inner
                .inspect(geometry, env, filter, limits)?
                .result
                .into_iter()
                .map(|r| {
                    r.map(|r| WithFiniteLoopInspectionResult {
                        inner: r,
                        segment: self.segment,
                        transition_mode: self.transition_mode,
                        loop_count: self.loop_count,
                    })
                })
                .collect(),
        })
    }
}
