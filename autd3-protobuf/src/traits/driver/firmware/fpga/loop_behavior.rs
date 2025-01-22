use std::num::NonZeroU16;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::firmware::fpga::LoopBehavior {
    type Message = LoopBehavior;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            rep: self.rep() as _,
        })
    }
}

impl FromMessage<LoopBehavior> for autd3_driver::firmware::fpga::LoopBehavior {
    fn from_msg(msg: &LoopBehavior) -> Result<Self, AUTDProtoBufError> {
        Ok(match msg.rep {
            0xFFFF => autd3_driver::firmware::fpga::LoopBehavior::Infinite,
            v => u16::try_from(v)
                .map(|v| NonZeroU16::new(v + 1).unwrap())
                .map(autd3_driver::firmware::fpga::LoopBehavior::Finite)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::LoopBehavior;
    use rand::Rng;

    #[test]
    fn test_loop_behavior() {
        {
            let mut rng = rand::thread_rng();
            let v = LoopBehavior::Finite(NonZeroU16::new(rng.gen_range(1..=0xFFFF)).unwrap());
            let msg = v.to_msg(None).unwrap();
            let v2 = LoopBehavior::from_msg(&msg).unwrap();
            assert_eq!(v, v2);
        }

        {
            let v = LoopBehavior::Infinite;
            let msg = v.to_msg(None).unwrap();
            let v2 = LoopBehavior::from_msg(&msg).unwrap();
            assert_eq!(v, v2);
        }
    }
}
