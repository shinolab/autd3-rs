use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    environment::Environment,
    firmware::FirmwareLimits,
    geometry::Geometry,
};

/// [`Datagram`] to synchronize the devices.
#[derive(Default, Debug)]
pub struct Synchronize {}

impl Synchronize {
    /// Creates a new [`Synchronize`] instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Datagram<'_> for Synchronize {
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
