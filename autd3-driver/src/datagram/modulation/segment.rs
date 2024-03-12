use std::time::Duration;

use crate::{common::Segment, error::AUTDInternalError};

use super::Datagram;

#[derive(Debug, Clone, Copy)]
pub struct ChangeModulationSegment {
    segment: Segment,
}

impl ChangeModulationSegment {
    pub const fn new(segment: Segment) -> Self {
        Self { segment }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl Datagram for ChangeModulationSegment {
    type O1 = crate::operation::ModulationChangeSegmentOp;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.segment), Self::O2::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_mod_segment() -> anyhow::Result<()> {
        let d = ChangeModulationSegment::new(Segment::S0);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(Duration::from_millis(200)), d.timeout());
        let _ = d.operation()?;
        Ok(())
    }
}
