mod cpu_gpio;
mod filter;
mod fpga_gpio;
mod inspect;
mod limits;
mod loop_behavior;
mod option;
mod pulse_width;
mod segment;
mod transition_mode;
mod tuple;

pub use cpu_gpio::CpuGPIOPort;
pub use fpga_gpio::{GPIOIn, GPIOOut};
pub use inspect::{Inspectable, InspectionResult};
pub use limits::FirmwareLimits;
pub use loop_behavior::LoopBehavior;
pub use pulse_width::PulseWidth;
pub use segment::Segment;
pub use transition_mode::{TRANSITION_MODE_NONE, TransitionMode};
pub use tuple::{CombinedError, CombinedOperationGenerator};

pub use filter::DeviceFilter;
pub use option::DatagramOption;

use crate::{derive::Environment, geometry::Geometry};

/// [`DatagramL`] is a [`Datagram`] with [`LoopBehavior`].
pub trait DatagramL: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    fn operation_generator_with_loop_behavior(
        self,
        geometry: &Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
        loop_behavior: LoopBehavior,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    #[must_use]
    fn option(&self) -> DatagramOption;
}

/// [`DatagramS`] is a [`Datagram`] with [`Segment`].
pub trait DatagramS: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    #[must_use]
    fn option(&self) -> DatagramOption;
}

impl<D: DatagramL> DatagramS for D {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_loop_behavior(
            geometry,
            env,
            filter,
            limits,
            segment,
            transition_mode,
            LoopBehavior::Infinite,
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramL>::option(self)
    }
}

/// [`Datagram`] represents the data sent to the device.
pub trait Datagram: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn operation_generator(
        self,
        geometry: &Geometry,
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

impl<D: DatagramS> Datagram for D {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &Geometry,
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
            Some(TransitionMode::Immediate),
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(self)
    }
}
