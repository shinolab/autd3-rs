use super::{Datagram, DeviceFilter};
use crate::geometry::{Device, Geometry};

use derive_more::Deref;

/// Inspection result of a [`Datagram`].
#[derive(Clone, Deref)]
pub struct InspectionResult<T> {
    #[deref]
    /// The inspection result for each device.
    pub result: Vec<Option<T>>,
}

impl<T> InspectionResult<T> {
    #[must_use]
    #[doc(hidden)]
    pub fn new(
        geometry: &Geometry,
        filter: &DeviceFilter,
        mut f: impl FnMut(&Device) -> T,
    ) -> Self {
        Self {
            result: geometry
                .iter()
                .map(|dev| filter.is_enabled(dev).then(|| f(dev)))
                .collect(),
        }
    }

    #[must_use]
    #[doc(hidden)]
    pub fn modify(self, f: impl Fn(T) -> T) -> Self {
        Self {
            result: self.result.into_iter().map(|r| r.map(&f)).collect(),
        }
    }
}

/// Trait to inspect a [`Datagram`].
pub trait Inspectable: Datagram {
    /// The result of the inspection.
    type Result;

    /// Returns the inspection result.
    fn inspect(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
    ) -> Result<InspectionResult<Self::Result>, Self::Error>;
}
