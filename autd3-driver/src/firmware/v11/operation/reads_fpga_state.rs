use super::OperationGenerator;
use crate::{
    datagram::ReadsFPGAState,
    firmware::v10::operation::OperationGenerator as OperationGeneratorV10, geometry::Device,
};

impl<F: Fn(&Device) -> bool> OperationGenerator for ReadsFPGAState<F> {
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
