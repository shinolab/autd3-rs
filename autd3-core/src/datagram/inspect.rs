use super::{Datagram, DeviceMask};
use crate::{
    environment::Environment,
    geometry::{Device, Geometry},
};

use alloc::vec::Vec;

/// Inspection result of a [`Datagram`].
#[derive(Clone)]
pub struct InspectionResult<T> {
    /// The inspection result for each device.
    pub result: Vec<Option<T>>,
}

impl<T> InspectionResult<T> {
    #[must_use]
    #[doc(hidden)]
    pub fn new<'a>(
        geometry: &'a Geometry,
        filter: &DeviceMask,
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

impl<T> core::ops::Deref for InspectionResult<T> {
    type Target = [Option<T>];

    fn deref(&self) -> &Self::Target {
        &self.result
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
        filter: &DeviceMask,
    ) -> Result<InspectionResult<Self::Result>, Self::Error>;
}
