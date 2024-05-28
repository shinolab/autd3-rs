use crate::firmware::{fpga::GPIOIn, operation::EmulateGPIOInOp};

use crate::datagram::*;

pub struct EmulateGPIOIn<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H + Send + Sync> {
    f: F,
}

impl<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H + Send + Sync> EmulateGPIOIn<H, F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct EmulateGPIOInOpGenerator<
    H: Fn(GPIOIn) -> bool + Send + Sync,
    F: Fn(&Device) -> H + Send + Sync,
> {
    f: F,
}

impl<'a, H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H + Send + Sync> OperationGenerator
    for EmulateGPIOInOpGenerator<H, F>
{
    type O1 = EmulateGPIOInOp;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        let h = (self.f)(device);
        Ok((
            Self::O1::new([h(GPIOIn::I0), h(GPIOIn::I1), h(GPIOIn::I2), h(GPIOIn::I3)]),
            Self::O2::default(),
        ))
    }
}

impl<'a, H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H + Send + Sync + 'a> Datagram<'a>
    for EmulateGPIOIn<H, F>
{
    type O1 = EmulateGPIOInOp;
    type O2 = NullOp;
    type G = EmulateGPIOInOpGenerator<H, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(EmulateGPIOInOpGenerator { f: self.f })
    }
}
