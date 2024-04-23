use std::time::Duration;

use crate::datagram::Datagram;

use super::group::{AUTDInternalError, Segment};

#[derive(Debug, Clone, Copy)]
pub struct ChangeGainSegment {
    segment: Segment,
}

impl ChangeGainSegment {
    pub const fn new(segment: Segment) -> Self {
        Self { segment }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl Datagram for ChangeGainSegment {
    type O1 = crate::firmware::operation::GainChangeSegmentOp;
    type O2 = crate::firmware::operation::NullOp;

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
    fn test() -> anyhow::Result<()> {
        let d = ChangeGainSegment::new(Segment::S0);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(Duration::from_millis(200)), d.timeout());
        let _ = d.operation()?;

        Ok(())
    }
}
