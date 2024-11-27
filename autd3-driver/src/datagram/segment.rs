use crate::{
    datagram::*,
    derive::{AUTDInternalError, Geometry},
    firmware::operation::SwapSegmentOp,
};

use super::OperationGenerator;

pub struct SwapSegmentOpGenerator {
    segment: SwapSegment,
}

impl OperationGenerator for SwapSegmentOpGenerator {
    type O1 = SwapSegmentOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(self.segment), Self::O2::new())
    }
}

impl Datagram for SwapSegment {
    type G = SwapSegmentOpGenerator;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SwapSegmentOpGenerator { segment: self })
    }
}
