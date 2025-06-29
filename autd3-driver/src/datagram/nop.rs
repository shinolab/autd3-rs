use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    derive::FirmwareLimits,
    environment::Environment,
    geometry::Geometry,
};
use derive_more::Debug;

/// [`Datagram`] which does nothing.
#[derive(Debug)]
pub struct Nop;

impl Datagram for Nop {
    type G = Nop;
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
