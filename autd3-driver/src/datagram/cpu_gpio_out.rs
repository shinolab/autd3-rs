use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    firmware::{CpuGPIOPort, FirmwareLimits},
    geometry::{Device, Geometry},
};

#[derive(Debug)]
#[doc(hidden)]
pub struct CpuGPIOOutputs<F> {
    pub(crate) f: F,
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> CpuGPIOOutputs<F> {
    /// Creates a new [`CpuGPIOOutputs`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> Datagram<'_> for CpuGPIOOutputs<F> {
    type G = Self;
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
