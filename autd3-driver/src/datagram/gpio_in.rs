use crate::{datagram::*, derive::DEFAULT_TIMEOUT, firmware::fpga::GPIOIn, geometry::Device};

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

impl<'a, H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H + Send + Sync>
    OperationGenerator<'a> for EmulateGPIOInOpGenerator<H, F>
{
    type O1 = crate::firmware::operation::EmulateGPIOInOp;
    type O2 = crate::firmware::operation::NullOp;

    fn generate(&'a self, device: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
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
    type O1 = crate::firmware::operation::EmulateGPIOInOp;
    type O2 = crate::firmware::operation::NullOp;
    type G =  EmulateGPIOInOpGenerator<H, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &'a Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(EmulateGPIOInOpGenerator { f: self.f })
    }
}
