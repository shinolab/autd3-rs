use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TransitionMode},
        operation::{cast, Operation, TypeTag},
    },
    geometry::{Device, Geometry},
};

#[repr(C, align(2))]
struct ModulationUpdate {
    tag: TypeTag,
    segment: u8,
    transition_mode: u8,
    __padding: [u8; 5],
    transition_value: u64,
}

pub struct ModulationChangeSegmentOp {
    segment: Segment,
    transition_mode: TransitionMode,
    remains: HashMap<usize, usize>,
}

impl ModulationChangeSegmentOp {
    pub fn new(segment: Segment, transition_mode: TransitionMode) -> Self {
        Self {
            segment,
            transition_mode,
            remains: HashMap::new(),
        }
    }
}

impl Operation for ModulationChangeSegmentOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        *cast::<ModulationUpdate>(tx) = ModulationUpdate {
            tag: TypeTag::ModulationChangeSegment,
            segment: self.segment as u8,
            transition_mode: self.transition_mode.mode(),
            __padding: [0; 5],
            transition_value: self.transition_mode.value(),
        };

        Ok(std::mem::size_of::<ModulationUpdate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ModulationUpdate>()
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
