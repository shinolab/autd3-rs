use super::{Datagram, DeviceFilter};
use crate::{
    environment::Environment,
    firmware::FirmwareLimits,
    geometry::{Device, Geometry},
};

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
    pub fn new<'a>(
        geometry: &'a Geometry,
        filter: &DeviceFilter,
        mut f: impl FnMut(&'a Device) -> T,
    ) -> Self {
        Self {
            result: geometry
                .iter()
                .map(|dev| filter.is_enabled(dev).then(|| f(dev)))
                .collect(),
        }
    }
}

/// Trait to inspect a [`Datagram`].
pub trait Inspectable<'a>: Datagram<'a> {
    /// The result of the inspection.
    type Result;

    /// Returns the inspection result.
    fn inspect(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<InspectionResult<Self::Result>, Self::Error>;
}
