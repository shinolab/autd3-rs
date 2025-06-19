use super::OperationGenerator;
use crate::firmware::driver::NullOp;
use crate::{
    firmware::v11::operation::OperationGenerator as OperationGeneratorV11, geometry::Device,
};

use autd3_core::modulation::ModulationOperationGenerator;

impl OperationGenerator for ModulationOperationGenerator {
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV11::generate(self, device)
    }
}
