use super::OperationGenerator;
use crate::{
    datagram::CpuGPIOOutputs,
    firmware::v10::operation::OperationGenerator as OperationGeneratorV10, geometry::Device,
};

use autd3_core::datagram::CpuGPIOPort;

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> OperationGenerator for CpuGPIOOutputs<F> {
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
