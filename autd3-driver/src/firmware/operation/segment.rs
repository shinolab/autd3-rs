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
            SwapSegment::FociSTM(segment, transition) => {
                *cast::<SwapSegmentTWithTransition>(tx) = SwapSegmentTWithTransition {
                    tag: TypeTag::FociSTMSwapSegment,
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
            | SwapSegment::FociSTM(_, _)
            | SwapSegment::GainSTM(_, _) => size_of::<SwapSegmentTWithTransition>(),
        }
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        derive::{Segment, TransitionMode},
        ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
        geometry::tests::create_device,
    };

    use super::*;

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn gain() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentT>();

        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op = SwapSegmentOp::new(SwapSegment::Gain(Segment::S0));

        assert_eq!(size_of::<SwapSegmentT>(), op.required_size(&device));
        assert_eq!(Ok(size_of::<SwapSegmentT>()), op.pack(&device, &mut tx));
        assert_eq!(op.is_done(), true);
        assert_eq!(TypeTag::GainSwapSegment as u8, tx[0]);
        assert_eq!(Segment::S0 as u8, tx[1]);
    }

    #[test]
    fn modulation() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentTWithTransition>();

        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
            + std::time::Duration::from_nanos(0x0123456789ABCDEF);
        let transition_mode = TransitionMode::SysTime(sys_time);
        let mut op = SwapSegmentOp::new(SwapSegment::Modulation(Segment::S0, transition_mode));

        assert_eq!(
            size_of::<SwapSegmentTWithTransition>(),
            op.required_size(&device)
        );
        assert_eq!(
            Ok(size_of::<SwapSegmentTWithTransition>()),
            op.pack(&device, &mut tx)
        );
        assert_eq!(op.is_done(), true);
        assert_eq!(TypeTag::ModulationSwapSegment as u8, tx[0]);
        assert_eq!(Segment::S0 as u8, tx[1]);
        let mode = transition_mode.mode();
        let value = transition_mode.value();
        assert_eq!(mode, tx[2]);
        assert_eq!(value, u64::from_le_bytes(tx[8..].try_into().unwrap()));
    }

    #[test]
    fn foci_stm() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentTWithTransition>();

        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
            + std::time::Duration::from_nanos(0x0123456789ABCDEF);
        let transition_mode = TransitionMode::SysTime(sys_time);
        let mut op = SwapSegmentOp::new(SwapSegment::FociSTM(Segment::S0, transition_mode));

        assert_eq!(
            size_of::<SwapSegmentTWithTransition>(),
            op.required_size(&device)
        );
        assert_eq!(
            Ok(size_of::<SwapSegmentTWithTransition>()),
            op.pack(&device, &mut tx)
        );
        assert_eq!(op.is_done(), true);
        assert_eq!(TypeTag::FociSTMSwapSegment as u8, tx[0]);
        assert_eq!(Segment::S0 as u8, tx[1]);
        let mode = transition_mode.mode();
        let value = transition_mode.value();
        assert_eq!(mode, tx[2]);
        assert_eq!(value, u64::from_le_bytes(tx[8..].try_into().unwrap()));
    }

    #[test]
    fn gain_stm() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentTWithTransition>();

        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
            + std::time::Duration::from_nanos(0x0123456789ABCDEF);
        let transition_mode = TransitionMode::SysTime(sys_time);
        let mut op = SwapSegmentOp::new(SwapSegment::GainSTM(Segment::S0, transition_mode));

        assert_eq!(
            size_of::<SwapSegmentTWithTransition>(),
            op.required_size(&device)
        );
        assert_eq!(
            Ok(size_of::<SwapSegmentTWithTransition>()),
            op.pack(&device, &mut tx)
        );
        assert_eq!(op.is_done(), true);
        assert_eq!(TypeTag::GainSTMSwapSegment as u8, tx[0]);
        assert_eq!(Segment::S0 as u8, tx[1]);
        let mode = transition_mode.mode();
        let value = transition_mode.value();
        assert_eq!(mode, tx[2]);
        assert_eq!(value, u64::from_le_bytes(tx[8..].try_into().unwrap()));
    }
}
