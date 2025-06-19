use super::OperationGenerator;
use crate::{
    datagram::PhaseCorrection,
    firmware::v11::operation::OperationGenerator as OperationGeneratorV11,
    geometry::{Device, Transducer},
};

use autd3_core::gain::Phase;

impl<FT: Fn(&Transducer) -> Phase + Send + Sync, F: Fn(&Device) -> FT> OperationGenerator
    for PhaseCorrection<FT, F>
{
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = <Self as OperationGeneratorV11>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV11::generate(self, device)
    }
}
