use super::super::OperationGenerator;
use crate::{
    datagram::{FociSTMIteratorGenerator, FociSTMOperationGenerator},
    firmware::v10::operation::OperationGenerator as OperationGeneratorV10,
    geometry::Device,
};

impl<const N: usize, G: FociSTMIteratorGenerator<N>> OperationGenerator
    for FociSTMOperationGenerator<N, G>
{
    type O1 = <Self as OperationGeneratorV10>::O1;
    type O2 = <Self as OperationGeneratorV10>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV10::generate(self, device)
    }
}
