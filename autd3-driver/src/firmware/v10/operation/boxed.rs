use super::OperationGenerator;
use crate::firmware::driver::{BoxedOperation, DynOperationGenerator, Version};

use autd3_core::geometry::Device;

impl<'a> OperationGenerator<'a> for DynOperationGenerator {
    type O1 = BoxedOperation;
    type O2 = BoxedOperation;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        self.g.dyn_generate(device, Version::V10)
    }
}
