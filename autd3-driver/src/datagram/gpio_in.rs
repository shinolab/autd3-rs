use std::convert::Infallible;

use crate::geometry::Device;

use autd3_core::{
    datagram::{Datagram, DeviceFilter, GPIOIn},
    derive::FirmwareLimits,
    geometry::Geometry,
};
use derive_more::Debug;

#[doc(hidden)]
#[derive(Debug)]
pub struct EmulateGPIOIn<F> {
    #[debug(ignore)]
    pub(crate) f: F,
}

impl<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H> EmulateGPIOIn<F> {
    /// Creates a new [`EmulateGPIOIn`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H> Datagram for EmulateGPIOIn<F> {
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &DeviceFilter,
        _: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
