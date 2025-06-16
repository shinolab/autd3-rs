use super::{Operation, OperationGenerator};

use crate::{
    datagram::SwapSegment,
    error::AUTDDriverError,
    firmware::v10::operation::{
        Operation as OperationV10, OperationGenerator as OperationGeneratorV10, SwapSegmentOp,
    },
    geometry::Device,
};

impl Operation for SwapSegmentOp {
    type Error = AUTDDriverError;

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

impl OperationGenerator for SwapSegment {
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
