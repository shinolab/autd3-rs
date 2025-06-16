use super::super::{Operation, OperationGenerator};
use crate::{
    datagram::FixedUpdateRate,
    firmware::v11::operation::{
        Operation as OperationV11, OperationGenerator as OperationGeneratorV11,
        SilencerFixedUpdateRateOp,
    },
    geometry::Device,
};

impl Operation for SilencerFixedUpdateRateOp {
    type Error = <Self as OperationV11>::Error;

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

impl OperationGenerator for FixedUpdateRate {
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = <Self as OperationGeneratorV11>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV11::generate(self, device)
    }
}
