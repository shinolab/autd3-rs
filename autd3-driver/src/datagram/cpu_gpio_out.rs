use crate::firmware::operation::CpuGPIOOutOp;

use crate::datagram::*;

use derive_more::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuGPIOPort {
    pub pa5: bool,
    pub pa7: bool,
}

#[derive(Debug)]
pub struct CpuGPIO<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> {
    #[debug(ignore)]
    f: F,
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> CpuGPIO<F> {
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

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        let port = (self.f)(device);
        (CpuGPIOOutOp::new(port.pa5, port.pa7), Self::O2::default())
    }
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> Datagram for CpuGPIO<F> {
    type G = CpuGPIOOutOpGenerator<F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(Self::G { f: self.f })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}
