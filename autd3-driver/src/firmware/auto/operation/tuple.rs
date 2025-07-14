use super::OperationGenerator;
use crate::firmware::driver::{NullOp, Version};

use autd3_core::{datagram::CombinedOperationGenerator, geometry::Device};

impl<'a, O1, O2> OperationGenerator<'a> for CombinedOperationGenerator<O1, O2>
where
    O1: OperationGenerator<'a, O2 = NullOp>,
    O2: OperationGenerator<'a, O2 = NullOp>,
{
    type O1 = O1::O1;
    type O2 = O2::O1;

    fn generate(&mut self, device: &'a Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        match (
            self.o1.generate(device, version),
            self.o2.generate(device, version),
        ) {
            (Some((o1, _)), Some((o2, _))) => Some((o1, o2)),
            _ => None,
        }
    }
}
