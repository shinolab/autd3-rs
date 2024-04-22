use std::time::Duration;

use crate::{derive::AUTDInternalError, fpga::Segment};

#[derive(Debug, Clone, Copy)]
pub struct ChangeFocusSTMSegment {
    segment: Segment,
}

impl ChangeFocusSTMSegment {
    pub const fn new(segment: Segment) -> Self {
        Self { segment }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl crate::datagram::Datagram for ChangeFocusSTMSegment {
    type O1 = crate::operation::FocusSTMChangeSegmentOp;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.segment), Self::O2::default()))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChangeGainSTMSegment {
    segment: Segment,
}

impl ChangeGainSTMSegment {
    pub const fn new(segment: Segment) -> Self {
        Self { segment }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl crate::datagram::Datagram for ChangeGainSTMSegment {
    type O1 = crate::operation::GainSTMChangeSegmentOp;
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
    fn test_focus_stm() -> anyhow::Result<()> {
        use crate::datagram::Datagram;
        let d = ChangeFocusSTMSegment::new(Segment::S0);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(Duration::from_millis(200)), d.timeout());
        let _ = d.operation()?;
        Ok(())
    }

    #[test]
    fn test_gain_stm() -> anyhow::Result<()> {
        use crate::datagram::Datagram;
        let d = ChangeGainSTMSegment::new(Segment::S0);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(Duration::from_millis(200)), d.timeout());
        let _ = d.operation()?;
        Ok(())
    }
}
