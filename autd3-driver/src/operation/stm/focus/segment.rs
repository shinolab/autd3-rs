use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    fpga::Segment,
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

#[repr(C, align(2))]
struct FocusSTMUpdate {
    tag: TypeTag,
    segment: u8,
}

pub struct FocusSTMChangeSegmentOp {
    segment: Segment,
    remains: HashMap<usize, usize>,
}

impl FocusSTMChangeSegmentOp {
    pub fn new(segment: Segment) -> Self {
        Self {
            segment,
            remains: HashMap::new(),
        }
    }
}

impl Operation for FocusSTMChangeSegmentOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        let d = cast::<FocusSTMUpdate>(tx);
        d.tag = TypeTag::FocusSTMChangeSegment;
        d.segment = self.segment as u8;

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
