use super::super::{Operation, OperationGenerator, null::NullOp};
use crate::{
    datagram::{GainSTMIterator, GainSTMIteratorGenerator, GainSTMOperationGenerator},
    firmware::v11::operation::{
        GainSTMOp, Operation as OperationV11, OperationGenerator as OperationGeneratorV11,
    },
    geometry::Device,
};

use autd3_core::gain::GainCalculator;

impl<G: GainCalculator, Iterator: GainSTMIterator<Calculator = G>> Operation
    for GainSTMOp<G, Iterator>
{
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

impl<T: GainSTMIteratorGenerator> OperationGenerator for GainSTMOperationGenerator<T> {
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV11::generate(self, device)
    }
}
