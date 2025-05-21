use std::convert::Infallible;

use crate::{
    datagram::*,
    firmware::{fpga::GPIOIn, operation::EmulateGPIOInOp},
};

use derive_more::Debug;

#[doc(hidden)]
#[derive(Debug)]
pub struct EmulateGPIOIn<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H> {
    #[debug(ignore)]
    f: F,
}

impl<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H> EmulateGPIOIn<H, F> {
    /// Creates a new [`EmulateGPIOIn`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct EmulateGPIOInOpGenerator<H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H> {
    f: F,
}

impl<H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H> OperationGenerator
    for EmulateGPIOInOpGenerator<H, F>
{
    type O1 = EmulateGPIOInOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        let h = (self.f)(device);
        (
            Self::O1::new([h(GPIOIn::I0), h(GPIOIn::I1), h(GPIOIn::I2), h(GPIOIn::I3)]),
            Self::O2 {},
        )
    }
}

impl<H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H> Datagram for EmulateGPIOIn<H, F> {
    type G = EmulateGPIOInOpGenerator<H, F>;
    type Error = Infallible;

    fn operation_generator(self, _: &mut Geometry) -> Result<Self::G, Self::Error> {
        Ok(EmulateGPIOInOpGenerator { f: self.f })
    }
}
