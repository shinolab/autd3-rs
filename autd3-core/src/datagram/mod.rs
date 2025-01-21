mod gpio;
mod loop_behavior;
mod operation;
mod segment;
mod transition_mode;
mod tuple;

pub use gpio::{GPIOIn, GPIOOut};
pub use loop_behavior::LoopBehavior;
pub use operation::{NullOp, Operation};
pub use segment::Segment;
pub use transition_mode::{TransitionMode, TRANSITION_MODE_NONE};
pub use tuple::{CombinedError, CombinedOperationGenerator};

use std::time::Duration;

use crate::{defined::DEFAULT_TIMEOUT, geometry::Geometry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatagramOption {
    pub timeout: Duration,
    pub parallel_threshold: usize,
}

impl Default for DatagramOption {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            parallel_threshold: usize::MAX,
        }
    }
}

/// [`DatagramL`] represents the data sent to the device.
pub trait DatagramL: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error: std::error::Error;

    #[doc(hidden)]
    fn operation_generator_with_loop_behavior(
        self,
        geometry: &Geometry,
        option: &DatagramOption,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
        loop_behavior: LoopBehavior,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    fn option(&self) -> DatagramOption {
        DatagramOption::default()
    }
}

/// [`DatagramS`] represents the data sent to the device.
pub trait DatagramS: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error: std::error::Error;

    #[doc(hidden)]
    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        option: &DatagramOption,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    fn option(&self) -> DatagramOption {
        DatagramOption::default()
    }
}

impl<D: DatagramL> DatagramS for D {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        option: &DatagramOption,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_loop_behavior(
            geometry,
            option,
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
    type Error: std::error::Error;

    #[doc(hidden)]
    fn operation_generator(
        self,
        geometry: &Geometry,
        option: &DatagramOption,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
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
        option: &DatagramOption,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_segment(
            geometry,
            option,
            Segment::S0,
            Some(TransitionMode::Immediate),
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(self)
    }
}
