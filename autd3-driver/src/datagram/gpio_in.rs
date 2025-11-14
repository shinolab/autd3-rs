use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    firmware::GPIOIn,
    geometry::{Device, Geometry},
};

#[doc(hidden)]
#[derive(Debug)]
pub struct EmulateGPIOIn<F, H> {
    pub(crate) f: F,
    _h: std::marker::PhantomData<H>,
}

impl<'a, H: Fn(GPIOIn) -> bool, F: Fn(&'a Device) -> H> EmulateGPIOIn<F, H> {
    /// Creates a new [`EmulateGPIOIn`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _h: std::marker::PhantomData,
        }
    }
}

impl<'a, H: Fn(GPIOIn) -> bool, F: Fn(&'a Device) -> H> Datagram<'_> for EmulateGPIOIn<F, H> {
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
