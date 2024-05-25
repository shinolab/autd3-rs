use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TransitionMode},
        operation::{cast, SwapSegmentOperation, TypeTag},
    },
    geometry::Device,
};

#[repr(C, align(2))]
struct FocusSTMUpdate {
    tag: TypeTag,
    segment: u8,
    transition_mode: u8,
    __padding: [u8; 5],
    transition_value: u64,
}

pub struct FocusSTMSwapSegmentOp {
    segment: Segment,
    transition_mode: TransitionMode,
    is_done: bool,
}

impl SwapSegmentOperation for FocusSTMSwapSegmentOp {
    fn new(segment: Segment, transition_mode: TransitionMode) -> Self {
        Self {
            segment,
            transition_mode,
            is_done: false,
        }
    }

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<FocusSTMUpdate>(tx) = FocusSTMUpdate {
            tag: TypeTag::FocusSTMSwapSegment,
            segment: self.segment as u8,
            transition_mode: self.transition_mode.mode(),
            __padding: [0; 5],
            transition_value: self.transition_mode.value(),
        };

        self.is_done = true;
        Ok(std::mem::size_of::<FocusSTMUpdate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<FocusSTMUpdate>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}
