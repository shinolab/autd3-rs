use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TransitionMode},
        operation::{cast, Operation, Remains, TypeTag},
    },
    geometry::{Device, Geometry},
};

#[repr(C)]
struct GainSTMUpdate {
    tag: TypeTag,
    segment: u8,
    transition_mode: u8,
    __padding: [u8; 5],
    transition_value: u64,
}

pub struct GainSTMChangeSegmentOp {
    segment: Segment,
    transition_mode: TransitionMode,
    remains: Remains,
}

impl GainSTMChangeSegmentOp {
    pub fn new(segment: Segment, transition_mode: TransitionMode) -> Self {
        Self {
            segment,
            transition_mode,
            remains: Default::default(),
        }
    }
}

impl Operation for GainSTMChangeSegmentOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<GainSTMUpdate>(tx) = GainSTMUpdate {
            tag: TypeTag::GainSTMChangeSegment,
            segment: self.segment as u8,
            transition_mode: self.transition_mode.mode(),
            __padding: [0; 5],
            transition_value: self.transition_mode.value(),
        };

        self.remains.send(device, 1);
        Ok(std::mem::size_of::<GainSTMUpdate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<GainSTMUpdate>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, 1);
        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}
