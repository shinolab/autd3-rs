use crate::firmware::{fpga::GPIOIn, operation::EmulateGPIOInOp};

use crate::datagram::*;

pub struct EmulateGPIOIn<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H> {
    f: F,
}

impl<H: Fn(GPIOIn) -> bool, F: Fn(&Device) -> H> EmulateGPIOIn<H, F> {
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

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        let h = (self.f)(device);
        (
            Self::O1::new([h(GPIOIn::I0), h(GPIOIn::I1), h(GPIOIn::I2), h(GPIOIn::I3)]),
            Self::O2::default(),
        )
    }
}

impl<H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H> Datagram for EmulateGPIOIn<H, F> {
    type O1 = EmulateGPIOInOp;
    type O2 = NullOp;
    type G = EmulateGPIOInOpGenerator<H, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(EmulateGPIOInOpGenerator { f: self.f })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }

    #[tracing::instrument(level = "debug", skip(self, geometry))]
    fn trace(&self, geometry: &Geometry) {
        tracing::info!("{}", tynm::type_name::<Self>());
        if tracing::enabled!(tracing::Level::DEBUG) {
            geometry.devices().for_each(|dev| {
                let f = (self.f)(dev);
                tracing::debug!(
                    "Device[{}]: I0={}, I1={}, I2={}, I3={}",
                    dev.idx(),
                    f(GPIOIn::I0),
                    f(GPIOIn::I1),
                    f(GPIOIn::I2),
                    f(GPIOIn::I3)
                )
            });
        }
    }
}
