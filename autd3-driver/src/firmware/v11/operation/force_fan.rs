use super::{Operation, OperationGenerator};
use crate::{
    datagram::ForceFan,
    firmware::v10::operation::{
        ForceFanOp, Operation as OperationV10, OperationGenerator as OperationGeneratorV10,
    },
    geometry::Device,
};

impl Operation for ForceFanOp {
    type Error = <Self as OperationV10>::Error;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        OperationV10::pack(self, device, tx)
    }

    fn required_size(&self, device: &Device) -> usize {
        OperationV10::required_size(self, device)
    }

    fn is_done(&self) -> bool {
        OperationV10::is_done(self)
    }
}

impl<F: Fn(&Device) -> bool> OperationGenerator for ForceFan<F> {
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
