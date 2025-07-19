use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceFilter, FirmwareLimits},
    environment::Environment,
    geometry::{Device, Geometry},
};

use derive_more::Debug;

/// [`Datagram`] to force the fan to run.
#[derive(Debug)]
pub struct ForceFan<F> {
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

impl<F: Fn(&Device) -> bool> Datagram<'_> for ForceFan<F> {
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &Environment,
        _: &DeviceFilter,
        _: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
