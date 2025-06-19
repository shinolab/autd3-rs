use autd3_core::datagram::GPIOIn;

use super::OperationGenerator;
use crate::{
    datagram::EmulateGPIOIn, firmware::v10::operation::OperationGenerator as OperationGeneratorV10,
    geometry::Device,
};

impl<H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H> OperationGenerator
    for EmulateGPIOIn<H, F>
{
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
