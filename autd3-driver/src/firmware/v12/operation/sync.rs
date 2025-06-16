use std::convert::Infallible;

use super::{Operation, OperationGenerator};
use crate::{
    datagram::Synchronize,
    firmware::v11::operation::{
        Operation as OperationV11, OperationGenerator as OperationGeneratorV11, SyncOp,
    },
    geometry::Device,
};

impl Operation for SyncOp {
    type Error = Infallible;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        OperationV11::pack(self, device, tx)
    }

    fn required_size(&self, device: &Device) -> usize {
        OperationV11::required_size(self, device)
    }

    fn is_done(&self) -> bool {
        OperationV11::is_done(self)
    }
}

impl OperationGenerator for Synchronize {
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = <Self as OperationGeneratorV11>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV11::generate(self, device)
    }
}
