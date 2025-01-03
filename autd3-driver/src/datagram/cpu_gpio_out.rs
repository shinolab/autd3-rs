use crate::firmware::operation::CpuGPIOOutOp;

use crate::datagram::*;

use derive_more::Debug;
use derive_new::new;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct CpuGPIOPort {
    pub pa5: bool,
    pub pa7: bool,
}

#[derive(Debug, new)]
#[doc(hidden)]
pub struct CpuGPIO<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> {
    #[debug(ignore)]
    f: F,
}

pub struct CpuGPIOOutOpGenerator<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> {
    f: F,
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> OperationGenerator for CpuGPIOOutOpGenerator<F> {
    type O1 = CpuGPIOOutOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        let port = (self.f)(device);
        (CpuGPIOOutOp::new(port.pa5, port.pa7), Self::O2::new())
    }
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> Datagram for CpuGPIO<F> {
    type G = CpuGPIOOutOpGenerator<F>;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDDriverError> {
        Ok(Self::G { f: self.f })
    }
}
