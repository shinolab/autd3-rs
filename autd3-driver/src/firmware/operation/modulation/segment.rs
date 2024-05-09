use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TransitionMode},
        operation::{cast, Remains, SwapSegmentOperation, TypeTag},
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

pub struct ModulationSwapSegmentOp {
    segment: Segment,
    transition_mode: TransitionMode,
    remains: Remains,
}

impl SwapSegmentOperation for ModulationSwapSegmentOp {
    fn new(segment: Segment, transition_mode: TransitionMode) -> Self {
        Self {
            segment,
            transition_mode,
            remains: Default::default(),
        }
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, |_| 1);
        Ok(())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ModulationUpdate>()
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<ModulationUpdate>(tx) = ModulationUpdate {
            tag: TypeTag::ModulationSwapSegment,
            segment: self.segment as u8,
            transition_mode: self.transition_mode.mode(),
            __padding: [0; 5],
            transition_value: self.transition_mode.value(),
        };

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<ModulationUpdate>())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}
