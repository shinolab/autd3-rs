use crate::pb::*;

impl From<Segment> for autd3_driver::common::Segment {
    fn from(value: Segment) -> Self {
        match value {
            Segment::S0 => Self::S0,
            Segment::S1 => Self::S1,
        }
    }
}

impl From<autd3_driver::common::Segment> for Segment {
    fn from(value: autd3_driver::common::Segment) -> Self {
        match value {
            autd3_driver::common::Segment::S0 => Self::S0,
            autd3_driver::common::Segment::S1 => Self::S1,
        }
    }
}
