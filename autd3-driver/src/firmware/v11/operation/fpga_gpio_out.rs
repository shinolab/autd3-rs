use super::OperationGenerator;
use crate::{
    datagram::{GPIOOutputType, GPIOOutputs},
    firmware::v10::operation::OperationGenerator as OperationGeneratorV10,
    geometry::Device,
};

use autd3_core::datagram::GPIOOut;

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> OperationGenerator
    for GPIOOutputs<F>
{
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
