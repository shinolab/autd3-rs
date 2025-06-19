use std::mem::size_of;

use super::OperationGenerator;

use crate::{
    datagram::SwapSegment,
    error::AUTDDriverError,
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
    geometry::Device,
};

use autd3_core::datagram::TransitionMode;

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct SwapSegmentT {
    tag: TypeTag,
    segment: u8,
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct SwapSegmentTWithTransition {
    tag: TypeTag,
    segment: u8,
    transition_mode: u8,
    __: [u8; 5],
    transition_value: u64,
}

pub struct SwapSegmentOp {
    is_done: bool,
    segment: SwapSegment,
}

impl SwapSegmentOp {
    pub(crate) fn new(segment: SwapSegment) -> Self {
        Self {
            is_done: false,
            segment,
        }
    }
}

impl Operation for SwapSegmentOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        self.is_done = true;

        let tag = match self.segment {
            SwapSegment::Gain(_, _) => TypeTag::GainSwapSegment,
            SwapSegment::Modulation(_, _) => TypeTag::ModulationSwapSegment,
            SwapSegment::FociSTM(_, _) => TypeTag::FociSTMSwapSegment,
            SwapSegment::GainSTM(_, _) => TypeTag::GainSTMSwapSegment,
        };

        match self.segment {
            SwapSegment::Gain(segment, transition) => {
                if transition != TransitionMode::Immediate {
                    return Err(AUTDDriverError::InvalidTransitionMode);
                }
                crate::firmware::driver::write_to_tx(
                    tx,
                    SwapSegmentT {
                        tag,
                        segment: segment as u8,
                    },
                );

                Ok(size_of::<SwapSegmentT>())
            }
            SwapSegment::Modulation(segment, transition)
            | SwapSegment::FociSTM(segment, transition)
            | SwapSegment::GainSTM(segment, transition) => {
                crate::firmware::driver::write_to_tx(
                    tx,
                    SwapSegmentTWithTransition {
                        tag,
                        segment: segment as u8,
                        transition_mode: transition.mode(),
                        __: [0; 5],
                        transition_value: transition.value(),
                    },
                );
                Ok(size_of::<SwapSegmentTWithTransition>())
            }
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        match self.segment {
            SwapSegment::Gain(_, _) => size_of::<SwapSegmentT>(),
            SwapSegment::Modulation(_, _)
            | SwapSegment::FociSTM(_, _)
            | SwapSegment::GainSTM(_, _) => size_of::<SwapSegmentTWithTransition>(),
        }
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl OperationGenerator for SwapSegment {
    type O1 = SwapSegmentOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(*self), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use crate::ethercat::DcSysTime;
    use autd3_core::datagram::Segment;

    use super::*;

    #[test]
    fn gain() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentT>();

        let device = crate::autd3_device::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op = SwapSegmentOp::new(SwapSegment::Gain(Segment::S0, TransitionMode::Immediate));

        assert_eq!(size_of::<SwapSegmentT>(), op.required_size(&device));
        assert_eq!(Ok(size_of::<SwapSegmentT>()), op.pack(&device, &mut tx));
        assert!(op.is_done());
        assert_eq!(TypeTag::GainSwapSegment as u8, tx[0]);
        assert_eq!(Segment::S0 as u8, tx[1]);
    }

    #[test]
    fn gain_invalid_transition_mode() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentT>();

        let device = crate::autd3_device::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op = SwapSegmentOp::new(SwapSegment::Gain(Segment::S0, TransitionMode::Ext));

        assert_eq!(
            Some(AUTDDriverError::InvalidTransitionMode),
            op.pack(&device, &mut tx).err()
        );
    }

    #[test]
    fn modulation() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentTWithTransition>();

        let device = crate::autd3_device::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::ZERO + std::time::Duration::from_nanos(0x0123456789ABCDEF);
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
        assert!(op.is_done());
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

        let device = crate::autd3_device::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::ZERO + std::time::Duration::from_nanos(0x0123456789ABCDEF);
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
        assert!(op.is_done());
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

        let device = crate::autd3_device::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::ZERO + std::time::Duration::from_nanos(0x0123456789ABCDEF);
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
        assert!(op.is_done());
        assert_eq!(TypeTag::GainSTMSwapSegment as u8, tx[0]);
        assert_eq!(Segment::S0 as u8, tx[1]);
        let mode = transition_mode.mode();
        let value = transition_mode.value();
        assert_eq!(mode, tx[2]);
        assert_eq!(value, u64::from_le_bytes(tx[8..].try_into().unwrap()));
    }
}
