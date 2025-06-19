use super::OperationGenerator;
use crate::firmware::driver::{BoxedOperation, DynOperationGenerator, Version};

use autd3_core::geometry::Device;

impl OperationGenerator for DynOperationGenerator {
    type O1 = BoxedOperation;
    type O2 = BoxedOperation;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        self.g.dyn_generate(device, Version::V12)
    }
}
