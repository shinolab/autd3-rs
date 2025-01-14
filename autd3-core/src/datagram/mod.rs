mod gpio;
mod operation;
mod segment;
mod transition_mode;
mod tuple;

pub use gpio::{GPIOIn, GPIOOut};
pub use operation::{NullOp, Operation};
pub use segment::Segment;
pub use transition_mode::{TransitionMode, TRANSITION_MODE_NONE};
pub use tuple::{CombinedError, CombinedOperationGenerator};

use std::time::Duration;

use crate::{defined::DEFAULT_TIMEOUT, geometry::Geometry};

/// [`Datagram`] represents the data sent to the device.
pub trait Datagram: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error: std::error::Error;

    #[doc(hidden)]
    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, Self::Error>;

    /// Returns the timeout duration.
    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    /// Returns the parallel threshold.
    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}

/// [`DatagramS`] represents a [`Datagram`] that can specify [`Segment`] to write the data.
pub trait DatagramS: std::fmt::Debug {
    #[doc(hidden)]
    type G;
    #[doc(hidden)]
    type Error: std::error::Error;

    #[doc(hidden)]
    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error>;

    /// Returns the timeout duration.
    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    /// Returns the parallel threshold.
    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}

impl<D: DatagramS> Datagram for D {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, D::Error> {
        self.operation_generator_with_segment(
            geometry,
            Segment::S0,
            Some(TransitionMode::Immediate),
        )
    }

    fn timeout(&self) -> Option<Duration> {
        <Self as DatagramS>::timeout(self)
    }

    fn parallel_threshold(&self) -> Option<usize> {
        <Self as DatagramS>::parallel_threshold(self)
    }
}
