use crate::{
    derive::TransitionMode,
    error::AUTDInternalError,
    firmware::{
        fpga::Segment,
        operation::{cast, SwapSegmentOperation, TypeTag},
    },
    geometry::Device,
};

#[repr(C, align(2))]
struct GainSwapSegmen {
    tag: TypeTag,
    segment: u8,
}

pub struct GainSwapSegmentOp {
    segment: Segment,
    is_done: bool,
}

impl SwapSegmentOperation for GainSwapSegmentOp {
    fn new(segment: Segment, _: TransitionMode) -> Self {
        Self {
            segment,
            is_done: false,
        }
    }

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<GainSwapSegmen>(tx) = GainSwapSegmen {
            tag: TypeTag::GainSwapSegment,
            segment: self.segment as u8,
        };

        self.is_done = true;
        Ok(std::mem::size_of::<GainSwapSegmen>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<GainSwapSegmen>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}
