use super::super::{Operation, OperationGenerator, null::NullOp};
use crate::{
    datagram::{FociSTMIterator, FociSTMIteratorGenerator, FociSTMOperationGenerator},
    firmware::v10::operation::{
        FociSTMOp, Operation as OperationV10, OperationGenerator as OperationGeneratorV10,
    },
    geometry::Device,
};

impl<const N: usize, Iterator: FociSTMIterator<N>> Operation for FociSTMOp<N, Iterator> {
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

impl<const N: usize, G: FociSTMIteratorGenerator<N>> OperationGenerator
    for FociSTMOperationGenerator<N, G>
{
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
