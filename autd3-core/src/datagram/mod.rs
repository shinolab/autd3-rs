mod filter;
mod inspect;
mod option;
mod tuple;

pub use filter::DeviceMask;
pub use inspect::{Inspectable, InspectionResult};
pub use option::DatagramOption;
pub use tuple::{CombinedError, CombinedOperationGenerator};

use crate::{
    environment::Environment,
    firmware::Segment,
    firmware::transition_mode::{Immediate, TransitionMode, TransitionModeParams},
    geometry::Geometry,
};

#[doc(hidden)]
pub mod internal {
    #[doc(hidden)]
    pub trait HasSegment<T> {}
    #[doc(hidden)]
    pub trait HasFiniteLoop<T> {}
}

const INFINITE_REP: u16 = 0xFFFF;

/// [`DatagramL`] is a [`Datagram`] with finite loop behavior.
pub trait DatagramL<'a> {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    fn operation_generator_with_finite_loop(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        segment: Segment,
        transition_params: TransitionModeParams,
        rep: u16,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    #[must_use]
    fn option(&self) -> DatagramOption;
}

/// [`DatagramS`] is a [`Datagram`] with [`Segment`].
pub trait DatagramS<'a> {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn operation_generator_with_segment(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    #[must_use]
    fn option(&self) -> DatagramOption;
}

impl<'a, D: DatagramL<'a>> DatagramS<'a> for D {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator_with_segment(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_finite_loop(
            geometry,
            env,
            filter,
            segment,
            transition_params,
            INFINITE_REP,
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramL<'a>>::option(self)
    }
}

/// [`Datagram`] represents the data sent to the device.
pub trait Datagram<'a> {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn operation_generator(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    #[must_use]
    fn option(&self) -> DatagramOption {
        DatagramOption::default()
    }
}

impl<'a, D: DatagramS<'a>> Datagram<'a> for D {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_segment(
            geometry,
            env,
            filter,
            Segment::S0,
            Immediate.params(),
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(self)
    }
}
