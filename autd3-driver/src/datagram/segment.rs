use std::time::Duration;

use crate::firmware::operation::{
    FocusSTMSwapSegmentOp, GainSTMSwapSegmentOp, GainSwapSegmentOp, ModulationSwapSegmentOp,
    SwapSegmentOperation,
};

use crate::{
    datagram::*,
    derive::{AUTDInternalError, Device, Geometry, Segment, TransitionMode},
};

use super::OperationGenerator;

pub trait SwapSegmentDatagram {
    type O: SwapSegmentOperation;
}

pub struct Gain;
impl SwapSegmentDatagram for Gain {
    type O = GainSwapSegmentOp;
}

pub struct Modulation;
impl SwapSegmentDatagram for Modulation {
    type O = ModulationSwapSegmentOp;
}

pub struct FocusSTM;
impl SwapSegmentDatagram for FocusSTM {
    type O = FocusSTMSwapSegmentOp;
}

pub struct GainSTM;
impl SwapSegmentDatagram for GainSTM {
    type O = GainSTMSwapSegmentOp;
}

#[derive(Debug, Clone, Copy)]
pub struct SwapSegment<T> {
    segment: Segment,
    transition_mode: TransitionMode,
    _phantom: std::marker::PhantomData<T>,
}

impl SwapSegment<()> {
    pub const fn gain(segment: Segment) -> SwapSegment<Gain> {
        SwapSegment {
            segment,
            transition_mode: TransitionMode::Immediate,
            _phantom: std::marker::PhantomData,
        }
    }

    pub const fn modulation(
        segment: Segment,
        transition_mode: TransitionMode,
    ) -> SwapSegment<Modulation> {
        SwapSegment {
            segment,
            transition_mode,
            _phantom: std::marker::PhantomData,
        }
    }

    pub const fn focus_stm(
        segment: Segment,
        transition_mode: TransitionMode,
    ) -> SwapSegment<FocusSTM> {
        SwapSegment {
            segment,
            transition_mode,
            _phantom: std::marker::PhantomData,
        }
    }

    pub const fn gain_stm(
        segment: Segment,
        transition_mode: TransitionMode,
    ) -> SwapSegment<GainSTM> {
        SwapSegment {
            segment,
            transition_mode,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> SwapSegment<T> {
    pub const fn segment(&self) -> Segment {
        self.segment
    }

    pub const fn transition_mode(&self) -> TransitionMode {
        self.transition_mode
    }
}

pub struct SwapSegmentOpGenerator<T: SwapSegmentDatagram + Sync + Send> {
    _phantom: std::marker::PhantomData<T>,
    segment: Segment,
    transition_mode: TransitionMode,
}

impl<'a, T: SwapSegmentDatagram + Sync + Send + 'a> OperationGenerator
    for SwapSegmentOpGenerator<T>
{
    type O1 = T::O;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(self.segment, self.transition_mode),
            Self::O2::default(),
        )
    }
}

impl<'a, T: SwapSegmentDatagram + Sync + Send + 'a> Datagram<'a> for SwapSegment<T> {
    type O1 = T::O;
    type O2 = NullOp;
    type G = SwapSegmentOpGenerator<T>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SwapSegmentOpGenerator {
            _phantom: std::marker::PhantomData,
            segment: self.segment,
            transition_mode: self.transition_mode,
        })
    }
}
