use std::mem::size_of;

use crate::{
    datagram::{SwapSegmentFociSTM, SwapSegmentGain, SwapSegmentGainSTM, SwapSegmentModulation},
    error::AUTDDriverError,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::{
    firmware::{
        Segment,
        transition_mode::{TransitionMode, TransitionModeParams},
    },
    geometry::Device,
};

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
    __: [u8; 5],
    transition_value: u64,
}

pub struct SwapSegmentOp {
    is_done: bool,
    tag: TypeTag,
    segment: u8,
    transition_params: Option<TransitionModeParams>,
}

impl SwapSegmentOp {
    fn new(tag: TypeTag, segment: Segment) -> Self {
        Self {
            is_done: false,
            tag,
            segment: segment as u8,
            transition_params: None,
        }
    }

    fn with_transition(
        tag: TypeTag,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Self {
        Self {
            is_done: false,
            tag,
            segment: segment as u8,
            transition_params: Some(transition_params),
        }
    }
}

impl Operation<'_> for SwapSegmentOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        self.is_done = true;

        let Self {
            tag,
            segment,
            transition_params,
            ..
        } = self;

        if let Some(transition_params) = transition_params {
            crate::firmware::operation::write_to_tx(
                tx,
                SwapSegmentTWithTransition {
                    tag: *tag,
                    segment: *segment,
                    transition_mode: transition_params.mode,
                    __: [0; 5],
                    transition_value: transition_params.value,
                },
            );
            Ok(size_of::<SwapSegmentTWithTransition>())
        } else {
            crate::firmware::operation::write_to_tx(
                tx,
                SwapSegmentT {
                    tag: *tag,
                    segment: *segment,
                },
            );
            Ok(size_of::<SwapSegmentT>())
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        if self.transition_params.is_some() {
            size_of::<SwapSegmentTWithTransition>()
        } else {
            size_of::<SwapSegmentT>()
        }
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl OperationGenerator<'_> for SwapSegmentGain {
    type O1 = SwapSegmentOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(TypeTag::GainSwapSegment, self.0), Self::O2 {}))
    }
}

impl<T: TransitionMode> OperationGenerator<'_> for SwapSegmentModulation<T> {
    type O1 = SwapSegmentOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::with_transition(TypeTag::ModulationSwapSegment, self.0, self.1.params()),
            Self::O2 {},
        ))
    }
}

impl<T: TransitionMode> OperationGenerator<'_> for SwapSegmentFociSTM<T> {
    type O1 = SwapSegmentOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::with_transition(TypeTag::FociSTMSwapSegment, self.0, self.1.params()),
            Self::O2 {},
        ))
    }
}

impl<T: TransitionMode> OperationGenerator<'_> for SwapSegmentGainSTM<T> {
    type O1 = SwapSegmentOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::with_transition(TypeTag::GainSTMSwapSegment, self.0, self.1.params()),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::ethercat::DcSysTime;
    use autd3_core::firmware::{
        Segment,
        transition_mode::{self, TransitionMode},
    };

    use super::*;

    #[test]
    fn gain() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentT>();

        let device = crate::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op = SwapSegmentGain(Segment::S0).generate(&device).unwrap().0;

        assert_eq!(size_of::<SwapSegmentT>(), op.required_size(&device));
        assert_eq!(Ok(size_of::<SwapSegmentT>()), op.pack(&device, &mut tx));
        assert!(op.is_done());
        assert_eq!(TypeTag::GainSwapSegment as u8, tx[0]);
        assert_eq!(Segment::S0 as u8, tx[1]);
    }

    #[test]
    fn modulation() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentTWithTransition>();

        let device = crate::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::ZERO + std::time::Duration::from_nanos(0x0123456789ABCDEF);
        let transition_mode = transition_mode::SysTime(sys_time);
        let mut op = SwapSegmentModulation(Segment::S0, transition_mode)
            .generate(&device)
            .unwrap()
            .0;

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
        let mode = transition_mode.params().mode;
        let value = transition_mode.params().value;
        assert_eq!(mode, tx[2]);
        assert_eq!(value, u64::from_le_bytes(tx[8..].try_into().unwrap()));
    }

    #[test]
    fn foci_stm() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentTWithTransition>();

        let device = crate::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::ZERO + std::time::Duration::from_nanos(0x0123456789ABCDEF);
        let transition_mode = transition_mode::SysTime(sys_time);
        let mut op = SwapSegmentFociSTM(Segment::S0, transition_mode)
            .generate(&device)
            .unwrap()
            .0;

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
        let mode = transition_mode.params().mode;
        let value = transition_mode.params().value;
        assert_eq!(mode, tx[2]);
        assert_eq!(value, u64::from_le_bytes(tx[8..].try_into().unwrap()));
    }

    #[test]
    fn gain_stm() {
        const FRAME_SIZE: usize = size_of::<SwapSegmentTWithTransition>();

        let device = crate::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let sys_time = DcSysTime::ZERO + std::time::Duration::from_nanos(0x0123456789ABCDEF);
        let transition_mode = transition_mode::SysTime(sys_time);
        let mut op = SwapSegmentGainSTM(Segment::S0, transition_mode)
            .generate(&device)
            .unwrap()
            .0;

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
        let mode = transition_mode.params().mode;
        let value = transition_mode.params().value;
        assert_eq!(mode, tx[2]);
        assert_eq!(value, u64::from_le_bytes(tx[8..].try_into().unwrap()));
    }
}
