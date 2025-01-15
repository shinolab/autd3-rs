use std::convert::Infallible;

use crate::{datagram::*, firmware::operation::SwapSegmentOp};

use super::OperationGenerator;

pub struct SwapSegmentOpGenerator {
    segment: SwapSegment,
}

impl OperationGenerator for SwapSegmentOpGenerator {
    type O1 = SwapSegmentOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(self.segment), Self::O2 {})
    }
}

impl Datagram for SwapSegment {
    type G = SwapSegmentOpGenerator;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, Self::Error> {
        Ok(SwapSegmentOpGenerator { segment: self })
    }
}
