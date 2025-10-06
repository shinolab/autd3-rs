use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    geometry::Geometry,
};

/// [`Datagram`] to clear all data in the devices.
#[derive(Default, Debug)]
pub struct Clear {}

impl Clear {
    /// Creates a new [`Clear`].
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Datagram<'_> for Clear {
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &Environment,
        _: &DeviceMask,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
