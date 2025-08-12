use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    firmware::FirmwareLimits,
    geometry::Geometry,
};
use derive_more::Debug;

/// [`Datagram`] which does nothing.
#[derive(Debug)]
pub struct Nop;

impl Datagram<'_> for Nop {
    type G = Nop;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &Environment,
        _: &DeviceMask,
        _: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
