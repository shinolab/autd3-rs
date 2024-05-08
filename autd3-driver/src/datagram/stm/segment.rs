use std::time::Duration;

use crate::{
    defined::DEFAULT_TIMEOUT,
    firmware::fpga::{Segment, TransitionMode},
};

#[derive(Debug, Clone, Copy)]
pub struct ChangeFocusSTMSegment {
    segment: Segment,
    transition_mode: TransitionMode,
}

impl ChangeFocusSTMSegment {
    pub fn new(segment: Segment, transition_mode: TransitionMode) -> Self {
        Self {
            segment,
            transition_mode,
        }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }

    pub const fn transition_mode(&self) -> TransitionMode {
        self.transition_mode
    }
}

impl crate::datagram::Datagram for ChangeFocusSTMSegment {
    type O1 = crate::firmware::operation::FocusSTMChangeSegmentOp;
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

#[derive(Debug, Clone, Copy)]
pub struct ChangeGainSTMSegment {
    segment: Segment,
    transition_mode: TransitionMode,
}

impl ChangeGainSTMSegment {
    pub fn new(segment: Segment, transition_mode: TransitionMode) -> Self {
        Self {
            segment,
            transition_mode,
        }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }

    pub const fn transition_mode(&self) -> TransitionMode {
        self.transition_mode
    }
}

impl crate::datagram::Datagram for ChangeGainSTMSegment {
    type O1 = crate::firmware::operation::GainSTMChangeSegmentOp;
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
    fn test_focus_stm() {
        use crate::datagram::Datagram;
        let d = ChangeFocusSTMSegment::new(Segment::S0, TransitionMode::Immidiate);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(TransitionMode::Immidiate, d.transition_mode());
        assert_eq!(Some(DEFAULT_TIMEOUT), d.timeout());
        let _ = d.operation();
    }

    #[test]
    fn test_gain_stm() {
        use crate::datagram::Datagram;
        let d = ChangeGainSTMSegment::new(Segment::S0, TransitionMode::Immidiate);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(TransitionMode::Immidiate, d.transition_mode());
        assert_eq!(Some(DEFAULT_TIMEOUT), d.timeout());
        let _ = d.operation();
    }
}
