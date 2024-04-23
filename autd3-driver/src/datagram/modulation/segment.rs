use std::time::Duration;

use crate::{
    error::AUTDInternalError,
    fpga::{Segment, TransitionMode},
};

use super::Datagram;

#[derive(Debug, Clone, Copy)]
pub struct ChangeModulationSegment {
    segment: Segment,
    transition_mode: TransitionMode,
}

impl ChangeModulationSegment {
    pub fn new(segment: Segment) -> Self {
        Self {
            segment,
            transition_mode: TransitionMode::default(),
        }
    }

    pub const fn with_transition_mode(self, transition_mode: TransitionMode) -> Self {
        Self {
            transition_mode,
            ..self
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
    type O1 = crate::operation::ModulationChangeSegmentOp;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((
            Self::O1::new(self.segment, self.transition_mode),
            Self::O2::default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let d = ChangeModulationSegment::new(Segment::S0);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(Duration::from_millis(200)), d.timeout());
        let _ = d.operation()?;
        Ok(())
    }
}
