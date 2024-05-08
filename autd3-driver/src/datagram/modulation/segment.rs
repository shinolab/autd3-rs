use std::time::Duration;

use crate::{
    defined::DEFAULT_TIMEOUT,
    firmware::fpga::{Segment, TransitionMode},
};

use super::Datagram;

#[derive(Debug, Clone, Copy)]
pub struct ChangeModulationSegment {
    segment: Segment,
    transition_mode: TransitionMode,
}

impl ChangeModulationSegment {
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

impl Datagram for ChangeModulationSegment {
    type O1 = crate::firmware::operation::ModulationChangeSegmentOp;
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
    fn test() -> anyhow::Result<()> {
        let d = ChangeModulationSegment::new(Segment::S0, TransitionMode::Immidiate);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(TransitionMode::Immidiate, d.transition_mode());
        assert_eq!(Some(DEFAULT_TIMEOUT), d.timeout());
        let _ = d.operation();
        Ok(())
    }
}
