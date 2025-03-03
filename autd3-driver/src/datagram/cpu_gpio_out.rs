use std::convert::Infallible;

use crate::firmware::operation::CpuGPIOOutOp;

use crate::datagram::*;

use derive_more::Debug;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuGPIOPort {
    pub pa5: bool,
    pub pa7: bool,
}

impl CpuGPIOPort {
    /// Creates a new [`CpuGPIOPort`].
    #[must_use]
    pub const fn new(pa5: bool, pa7: bool) -> Self {
        Self { pa5, pa7 }
    }
}

#[derive(Debug)]
#[doc(hidden)]
pub struct CpuGPIOOutputs<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> {
    #[debug(ignore)]
    f: F,
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> CpuGPIOOutputs<F> {
    /// Creates a new [`CpuGPIOOutputs`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct CpuGPIOOutOpGenerator<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> {
    f: F,
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> OperationGenerator for CpuGPIOOutOpGenerator<F> {
    type O1 = CpuGPIOOutOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        let port = (self.f)(device);
        (CpuGPIOOutOp::new(port.pa5, port.pa7), Self::O2 {})
    }
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> Datagram for CpuGPIOOutputs<F> {
    type G = CpuGPIOOutOpGenerator<F>;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: bool) -> Result<Self::G, Self::Error> {
        Ok(Self::G { f: self.f })
    }
}
