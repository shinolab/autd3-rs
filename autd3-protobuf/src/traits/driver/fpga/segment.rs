use crate::pb::*;

impl From<Segment> for autd3_driver::fpga::Segment {
    fn from(value: Segment) -> Self {
        match value {
            Segment::S0 => Self::S0,
            Segment::S1 => Self::S1,
        }
    }
}

impl From<autd3_driver::fpga::Segment> for Segment {
    fn from(value: autd3_driver::fpga::Segment) -> Self {
        match value {
            autd3_driver::fpga::Segment::S0 => Self::S0,
            autd3_driver::fpga::Segment::S1 => Self::S1,
        }
    }
}
