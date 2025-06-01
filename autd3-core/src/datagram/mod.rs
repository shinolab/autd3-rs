mod gpio;
mod inspect;
mod loop_behavior;
mod operation;
mod segment;
mod transition_mode;
mod tuple;

pub use gpio::{GPIOIn, GPIOOut};
pub use inspect::{Inspectable, InspectionResult};
pub use loop_behavior::LoopBehavior;
pub use operation::{NullOp, Operation};
pub use segment::Segment;
pub use transition_mode::{TRANSITION_MODE_NONE, TransitionMode};
pub use tuple::{CombinedError, CombinedOperationGenerator};

use std::time::Duration;

use crate::{
    common::DEFAULT_TIMEOUT,
    geometry::{Device, Geometry},
};

/// A filter that represents which devices are enabled.
pub struct DeviceFilter(pub(crate) Option<Vec<bool>>);

impl DeviceFilter {
    /// Returns a new `DeviceFilter` that enables all devices.
    pub const fn all_enabled() -> Self {
        Self(None)
    }

    /// Creates a `DeviceFilter` where the value at each index is `f(&Device)`
    pub fn from_fn(geo: &Geometry, f: impl Fn(&Device) -> bool) -> Self {
        Self(Some(geo.iter().map(f).collect()))
    }

    /// Returns `true` if the `Device` enabled.
    pub fn is_enabled(&self, dev: &Device) -> bool {
        self.0.as_ref().is_none_or(|f| f[dev.idx()])
    }

    /// Sets the device at `idx` to enabled.
    pub fn set_enable(&mut self, idx: usize) {
        if let Some(ref mut filter) = self.0 {
            filter[idx] = true;
        }
    }
}

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

impl DatagramOption {
    /// Merges two [`DatagramOption`]s.
    pub fn merge(self, other: DatagramOption) -> Self {
        Self {
            timeout: self.timeout.max(other.timeout),
            parallel_threshold: self.parallel_threshold.min(other.parallel_threshold),
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
        filter: &DeviceFilter,
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
        filter: &DeviceFilter,
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
        filter: &DeviceFilter,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_loop_behavior(
            geometry,
            filter,
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
        filter: &DeviceFilter,
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
        filter: &DeviceFilter,
    ) -> Result<Self::G, Self::Error> {
        self.operation_generator_with_segment(
            geometry,
            filter,
            Segment::S0,
            Some(TransitionMode::Immediate),
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramS>::option(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datagram_option_merge() {
        let opt1 = DatagramOption {
            timeout: Duration::from_secs(1),
            parallel_threshold: 10,
        };
        let opt2 = DatagramOption {
            timeout: Duration::from_secs(2),
            parallel_threshold: 5,
        };
        let opt3 = opt1.merge(opt2);
        assert_eq!(opt3.timeout, Duration::from_secs(2));
        assert_eq!(opt3.parallel_threshold, 5);
    }
}
