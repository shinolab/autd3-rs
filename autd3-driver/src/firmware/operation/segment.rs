use std::mem::size_of;

use crate::{
    datagram::SwapSegment,
    error::AUTDInternalError,
    firmware::operation::{cast, TypeTag},
    geometry::Device,
};

use super::Operation;

#[repr(C, align(2))]
struct SwapSegmentT {
    tag: TypeTag,
    segment: u8,
}

#[repr(C, align(2))]
struct SwapSegmentTWithTransition {
    tag: TypeTag,
    segment: u8,
    transition_mode: u8,
    __padding: [u8; 5],
    transition_value: u64,
}

pub struct SwapSegmentOp {
    segment: SwapSegment,
    is_done: bool,
}

impl SwapSegmentOp {
    pub fn new(segment: SwapSegment) -> Self {
        Self {
            segment,
            is_done: false,
        }
    }
}

impl Operation for SwapSegmentOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        self.is_done = true;

        match self.segment {
            SwapSegment::Gain(segment) => {
                *cast::<SwapSegmentT>(tx) = SwapSegmentT {
                    tag: TypeTag::GainSwapSegment,
                    segment: segment as u8,
                };
                Ok(size_of::<SwapSegmentT>())
            }
            SwapSegment::Modulation(segment, transition) => {
                *cast::<SwapSegmentTWithTransition>(tx) = SwapSegmentTWithTransition {
                    tag: TypeTag::ModulationSwapSegment,
                    segment: segment as u8,
                    transition_mode: transition.mode(),
                    __padding: [0; 5],
                    transition_value: transition.value(),
                };
                Ok(size_of::<SwapSegmentTWithTransition>())
            }
            SwapSegment::FocusSTM(segment, transition) => {
                *cast::<SwapSegmentTWithTransition>(tx) = SwapSegmentTWithTransition {
                    tag: TypeTag::FocusSTMSwapSegment,
                    segment: segment as u8,
                    transition_mode: transition.mode(),
                    __padding: [0; 5],
                    transition_value: transition.value(),
                };
                Ok(size_of::<SwapSegmentTWithTransition>())
            }
            SwapSegment::GainSTM(segment, transition) => {
                *cast::<SwapSegmentTWithTransition>(tx) = SwapSegmentTWithTransition {
                    tag: TypeTag::GainSTMSwapSegment,
                    segment: segment as u8,
                    transition_mode: transition.mode(),
                    __padding: [0; 5],
                    transition_value: transition.value(),
                };
                Ok(size_of::<SwapSegmentTWithTransition>())
            }
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        match self.segment {
            SwapSegment::Gain(_) => size_of::<SwapSegmentT>(),
            SwapSegment::Modulation(_, _)
            | SwapSegment::FocusSTM(_, _)
            | SwapSegment::GainSTM(_, _) => size_of::<SwapSegmentTWithTransition>(),
        }
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}
