mod cpu_gpio;
mod filter;
mod fpga_gpio;
mod inspect;
mod limits;
mod option;
mod pulse_width;
mod segment;
/// Transition odes for segment switching.
pub mod transition_mode;
mod tuple;

pub use cpu_gpio::CpuGPIOPort;
pub use fpga_gpio::{GPIOIn, GPIOOut};
pub use inspect::{Inspectable, InspectionResult};
pub use limits::FirmwareLimits;
pub use pulse_width::{PulseWidth, PulseWidthError};
pub use segment::Segment;
pub use tuple::{CombinedError, CombinedOperationGenerator};

pub use filter::DeviceFilter;
pub use option::DatagramOption;

use crate::{
    datagram::transition_mode::{Immediate, TransitionMode, TransitionModeParams},
    environment::Environment,
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

/// [`DatagramL`] is a [`Datagram`] with [`LoopBehavior`].
pub trait DatagramL<'a>: std::fmt::Debug {
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
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
        segment: Segment,
        transition_params: TransitionModeParams,
        rep: u16,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    #[must_use]
    fn option(&self) -> DatagramOption;
}

/// [`DatagramS`] is a [`Datagram`] with [`Segment`].
pub trait DatagramS<'a>: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn operation_generator_with_segment(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
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
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_finite_loop(
            geometry,
            env,
            filter,
            limits,
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
pub trait Datagram<'a>: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn operation_generator(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
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
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_segment(
            geometry,
            env,
            filter,
            limits,
            Segment::S0,
            Immediate.params(),
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(self)
    }
}
