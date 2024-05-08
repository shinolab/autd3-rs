use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TRANSITION_MODE_IMMIDIATE},
        operation::{cast, Operation, Remains, TypeTag},
    },
    geometry::{Device, Geometry},
};

#[repr(C, align(2))]
struct GainUpdate {
    tag: TypeTag,
    segment: u8,
    transition_mode: u8,
    __pad: u8,
}

pub struct GainChangeSegmentOp {
    segment: Segment,
    remains: Remains,
}

impl GainChangeSegmentOp {
    pub fn new(segment: Segment) -> Self {
        Self {
            segment,
            remains: Default::default(),
        }
    }
}

impl Operation for GainChangeSegmentOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<GainUpdate>(tx) = GainUpdate {
            tag: TypeTag::GainChangeSegment,
            segment: self.segment as u8,
            transition_mode: TRANSITION_MODE_IMMIDIATE,
            __pad: 0,
        };

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<GainUpdate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<GainUpdate>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, |_| 1);
        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}
