use crate::derive::{AUTDInternalError, Device, Segment, TransitionMode};

use super::Operation;

pub trait SwapSegmentOperation {
    fn new(segment: Segment, transition_mode: TransitionMode) -> Self;
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError>;
    fn is_done(&self) -> bool;
}

impl<T: SwapSegmentOperation> Operation for T {
    fn required_size(&self, device: &Device) -> usize {
        self.required_size(device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        self.pack(device, tx)
    }

    fn is_done(&self) -> bool {
        self.is_done()
    }
}
