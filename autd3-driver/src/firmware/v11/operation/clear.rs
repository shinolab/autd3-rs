use super::OperationGenerator;
use crate::{
    datagram::Clear, firmware::v10::operation::OperationGenerator as OperationGeneratorV10,
    geometry::Device,
};

impl OperationGenerator for Clear {
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
