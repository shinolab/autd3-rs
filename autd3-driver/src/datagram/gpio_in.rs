use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    firmware::GPIOIn,
    geometry::{Device, Geometry},
};

#[doc(hidden)]
#[derive(Debug)]
pub struct EmulateGPIOIn<F> {
    pub(crate) f: F,
}

impl<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H> EmulateGPIOIn<F> {
    /// Creates a new [`EmulateGPIOIn`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H> Datagram<'_> for EmulateGPIOIn<F> {
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
