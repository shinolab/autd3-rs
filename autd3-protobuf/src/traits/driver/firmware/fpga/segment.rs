use crate::{pb::*, FromMessage};

impl From<Segment> for autd3_driver::firmware::fpga::Segment {
    fn from(value: Segment) -> Self {
        match value {
            Segment::S0 => autd3_driver::firmware::fpga::Segment::S0,
            Segment::S1 => autd3_driver::firmware::fpga::Segment::S1,
        }
    }
}

impl From<autd3_driver::firmware::fpga::Segment> for Segment {
    fn from(value: autd3_driver::firmware::fpga::Segment) -> Self {
        match value {
            autd3_driver::firmware::fpga::Segment::S0 => Self::S0,
            autd3_driver::firmware::fpga::Segment::S1 => Self::S1,
        }
    }
}

impl FromMessage<i32> for autd3_driver::firmware::fpga::Segment {
    fn from_msg(msg: i32) -> Result<Self, crate::AUTDProtoBufError> {
        Ok(Segment::try_from(msg)?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment() {
        {
            let v = autd3_driver::firmware::fpga::Segment::S0;
            let msg: Segment = v.into();
            let v2: autd3_driver::firmware::fpga::Segment = msg.into();
            assert_eq!(v, v2);
        }

        {
            let v = autd3_driver::firmware::fpga::Segment::S1;
            let msg: Segment = v.into();
            let v2: autd3_driver::firmware::fpga::Segment = msg.into();
            assert_eq!(v, v2);
        }
    }
}
