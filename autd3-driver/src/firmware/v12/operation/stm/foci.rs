use super::super::OperationGenerator;
use crate::{
    datagram::{FociSTMIteratorGenerator, FociSTMOperationGenerator},
    firmware::v11::operation::OperationGenerator as OperationGeneratorV11,
    geometry::Device,
};

impl<const N: usize, G: FociSTMIteratorGenerator<N>> OperationGenerator
    for FociSTMOperationGenerator<N, G>
{
    type O1 = <Self as OperationGeneratorV11>::O1;
    type O2 = <Self as OperationGeneratorV11>::O2;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        OperationGeneratorV11::generate(self, device)
    }
}
