use std::convert::Infallible;

use crate::geometry::Device;

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    derive::FirmwareLimits,
    geometry::Geometry,
};

use derive_more::Debug;

/// [`Datagram`] to force the fan to run.
#[derive(Debug)]
pub struct ForceFan<F: Fn(&Device) -> bool> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
}

impl<F: Fn(&Device) -> bool> ForceFan<F> {
    /// Creates a new [`ForceFan`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn(&Device) -> bool> Datagram for ForceFan<F> {
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
