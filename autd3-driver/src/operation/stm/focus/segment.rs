use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    fpga::{Segment, TransitionMode},
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

#[repr(C, align(2))]
struct FocusSTMUpdate {
    tag: TypeTag,
    segment: u8,
    transition_mode: u8,
    __padding: [u8; 5],
    transition_value: u64,
}

pub struct FocusSTMChangeSegmentOp {
    segment: Segment,
    transition_mode: TransitionMode,
    remains: HashMap<usize, usize>,
}

impl FocusSTMChangeSegmentOp {
    pub fn new(segment: Segment, transition_mode: TransitionMode) -> Self {
        Self {
            segment,
            transition_mode,
            remains: HashMap::new(),
        }
    }
}

impl Operation for FocusSTMChangeSegmentOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        *cast::<FocusSTMUpdate>(tx) = FocusSTMUpdate {
            tag: TypeTag::FocusSTMChangeSegment,
            segment: self.segment as u8,
            transition_mode: self.transition_mode.mode(),
            __padding: [0; 5],
            transition_value: self.transition_mode.value(),
        };

        Ok(std::mem::size_of::<FocusSTMUpdate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<FocusSTMUpdate>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains = geometry.devices().map(|device| (device.idx(), 1)).collect();
        Ok(())
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }

    fn commit(&mut self, device: &Device) {
        self.remains.insert(device.idx(), 0);
    }
}
