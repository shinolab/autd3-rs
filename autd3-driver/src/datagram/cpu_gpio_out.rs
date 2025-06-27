use std::convert::Infallible;

use crate::geometry::Device;

use autd3_core::{
    datagram::{CpuGPIOPort, Datagram, DeviceFilter},
    derive::FirmwareLimits,
    geometry::Geometry,
};

use derive_more::Debug;

#[derive(Debug)]
#[doc(hidden)]
pub struct CpuGPIOOutputs<F> {
    #[debug(ignore)]
    pub(crate) f: F,
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> CpuGPIOOutputs<F> {
    /// Creates a new [`CpuGPIOOutputs`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> Datagram for CpuGPIOOutputs<F> {
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &DeviceFilter,
        _: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
