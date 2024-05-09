use crate::{
    derive::TransitionMode,
    error::AUTDInternalError,
    firmware::{
        fpga::Segment,
        operation::{cast, Remains, SwapSegmentOperation, TypeTag},
    },
    geometry::{Device, Geometry},
};

#[repr(C, align(2))]
struct GainUpdate {
    tag: TypeTag,
    segment: u8,
}

pub struct GainSwapSegmentOp {
    segment: Segment,
    remains: Remains,
}

impl SwapSegmentOperation for GainSwapSegmentOp {
    fn new(segment: Segment, _: TransitionMode) -> Self {
        Self {
            segment,
            remains: Default::default(),
        }
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<GainUpdate>(tx) = GainUpdate {
            tag: TypeTag::GainSwapSegment,
            segment: self.segment as u8,
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
