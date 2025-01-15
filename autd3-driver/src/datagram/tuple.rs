use autd3_core::{
    datagram::{CombinedOperationGenerator, NullOp},
    geometry::Device,
};

use crate::firmware::operation::OperationGenerator;

impl<O1, O2> OperationGenerator for CombinedOperationGenerator<O1, O2>
where
    O1: OperationGenerator,
    O2: OperationGenerator<O2 = NullOp>,
{
    type O1 = O1::O1;
    type O2 = O2::O1;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        let (o1, _) = self.o1.generate(device);
        let (o2, _) = self.o2.generate(device);
        (o1, o2)
    }
}
