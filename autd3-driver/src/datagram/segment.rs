use std::time::Duration;

use crate::{
    datagram::Datagram,
    defined::DEFAULT_TIMEOUT,
    derive::{Segment, TransitionMode},
    firmware::operation::SwapSegmentOperation,
};

pub trait SwapSegmentDatagram {
    type O: crate::firmware::operation::SwapSegmentOperation;
}

pub struct Gain;
impl SwapSegmentDatagram for Gain {
    type O = crate::firmware::operation::GainSwapSegmentOp;
}

pub struct Modulation;
impl SwapSegmentDatagram for Modulation {
    type O = crate::firmware::operation::ModulationSwapSegmentOp;
}

pub struct FocusSTM;
impl SwapSegmentDatagram for FocusSTM {
    type O = crate::firmware::operation::FocusSTMSwapSegmentOp;
}

pub struct GainSTM;
impl SwapSegmentDatagram for GainSTM {
    type O = crate::firmware::operation::GainSTMSwapSegmentOp;
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

impl<T: SwapSegmentDatagram> Datagram for SwapSegment<T> {
    type O1 = T::O;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(self) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(self.segment, self.transition_mode),
            Self::O2::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain() -> anyhow::Result<()> {
        let d = SwapSegment::gain(Segment::S0);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(DEFAULT_TIMEOUT), d.timeout());
        let _ = d.operation();

        Ok(())
    }

    #[test]
    fn modulation() -> anyhow::Result<()> {
        let d = SwapSegment::modulation(Segment::S0, TransitionMode::Immediate);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(TransitionMode::Immediate, d.transition_mode());
        assert_eq!(Some(DEFAULT_TIMEOUT), d.timeout());
        let _ = d.operation();
        Ok(())
    }

    #[test]
    fn focus_stm() {
        use crate::datagram::Datagram;
        let d = SwapSegment::focus_stm(Segment::S0, TransitionMode::Immediate);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(TransitionMode::Immediate, d.transition_mode());
        assert_eq!(Some(DEFAULT_TIMEOUT), d.timeout());
        let _ = d.operation();
    }

    #[test]
    fn gain_stm() {
        use crate::datagram::Datagram;
        let d = SwapSegment::gain_stm(Segment::S0, TransitionMode::Immediate);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(TransitionMode::Immediate, d.transition_mode());
        assert_eq!(Some(DEFAULT_TIMEOUT), d.timeout());
        let _ = d.operation();
    }
}
