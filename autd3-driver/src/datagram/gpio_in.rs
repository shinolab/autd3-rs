use crate::{datagram::*, derive::DEFAULT_TIMEOUT, firmware::fpga::GPIOIn, geometry::Device};

pub struct EmulateGPIOIn<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H + Send + Sync> {
    f: F,
}

impl<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H + Send + Sync> EmulateGPIOIn<H, F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<'a, H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H + Send + Sync> Datagram<'a>
    for EmulateGPIOIn<H, F>
{
    type O1 = crate::firmware::operation::EmulateGPIOInOp;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(
        &'a self,
        _: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        let f = &self.f;
        Ok(|dev| {
            let f = f(dev);
            (
                Self::O1::new([f(GPIOIn::I0), f(GPIOIn::I1), f(GPIOIn::I2), f(GPIOIn::I3)]),
                Self::O2::default(),
            )
        })
    }
}
