use super::OperationGenerator;
use crate::{
    datagram::FetchFirmwareInfoOpGenerator,
    firmware::v11::operation::OperationGenerator as OperationGeneratorV11, geometry::Device,
};

impl OperationGenerator for FetchFirmwareInfoOpGenerator {
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = <Self as OperationGeneratorV11>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV11::generate(self, device)
    }
}
