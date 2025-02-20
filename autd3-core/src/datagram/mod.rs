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
pub use transition_mode::{TRANSITION_MODE_NONE, TransitionMode};
pub use tuple::{CombinedError, CombinedOperationGenerator};

use std::time::Duration;

use crate::{defined::DEFAULT_TIMEOUT, geometry::Geometry};

/// The option of the datagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatagramOption {
    /// The default timeout of the datagram.
    pub timeout: Duration,
    /// The default threshold of the parallel processing.
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

/// [`DatagramL`] is a [`Datagram`] with [`LoopBehavior`].
pub trait DatagramL: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn operation_generator_with_loop_behavior(
        self,
        geometry: &Geometry,
        parallel: bool,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
        loop_behavior: LoopBehavior,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
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
        parallel: bool,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the option of the datagram.
    fn option(&self) -> DatagramOption;
}

impl<D: DatagramL> DatagramS for D {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        parallel: bool,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_loop_behavior(
            geometry,
            parallel,
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
        parallel: bool,
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
        parallel: bool,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_segment(
            geometry,
            parallel,
            Segment::S0,
            Some(TransitionMode::Immediate),
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(self)
    }
}
