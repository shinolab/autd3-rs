use super::{Operation, OperationGenerator};
use crate::{
    datagram::PhaseCorrection,
    firmware::v10::operation::{
        Operation as OperationV10, OperationGenerator as OperationGeneratorV10, PhaseCorrectionOp,
    },
    geometry::{Device, Transducer},
};

use autd3_core::gain::Phase;

impl<F: Fn(&Transducer) -> Phase + Send + Sync> Operation for PhaseCorrectionOp<F> {
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

impl<FT: Fn(&Transducer) -> Phase + Send + Sync, F: Fn(&Device) -> FT> OperationGenerator
    for PhaseCorrection<FT, F>
{
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
