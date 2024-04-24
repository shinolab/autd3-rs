use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::Segment,
        operation::{cast, Operation, TypeTag},
    },
    geometry::{Device, Geometry},
};

#[repr(C, align(2))]
struct GainUpdate {
    tag: TypeTag,
    segment: u8,
}

pub struct GainChangeSegmentOp {
    segment: Segment,
    remains: HashMap<usize, usize>,
}

impl GainChangeSegmentOp {
    pub fn new(segment: Segment) -> Self {
        Self {
            segment,
            remains: HashMap::new(),
        }
    }
}

impl Operation for GainChangeSegmentOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        let d = cast::<GainUpdate>(tx);
        d.tag = TypeTag::GainChangeSegment;
        d.segment = self.segment as u8;

        self.remains.insert(device.idx(), 0);
        Ok(std::mem::size_of::<GainUpdate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<GainUpdate>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains = geometry.devices().map(|device| (device.idx(), 1)).collect();
        Ok(())
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }
}
