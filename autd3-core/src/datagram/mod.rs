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
