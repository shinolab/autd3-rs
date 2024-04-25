use std::num::NonZeroU32;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::firmware::fpga::LoopBehavior {
    type Message = LoopBehavior;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            rep: match self {
                autd3_driver::firmware::fpga::LoopBehavior::Infinite => 0xFFFFFFFF,
                autd3_driver::firmware::fpga::LoopBehavior::Finite(n) => n.get() - 1,
            },
        }
    }
}

impl FromMessage<LoopBehavior> for autd3_driver::firmware::fpga::LoopBehavior {
    fn from_msg(msg: &LoopBehavior) -> Option<Self> {
        Some(match msg.rep {
            0xFFFFFFFF => autd3_driver::firmware::fpga::LoopBehavior::Infinite,
            v => {
                autd3_driver::firmware::fpga::LoopBehavior::Finite(NonZeroU32::new(v + 1).unwrap())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::derive::LoopBehavior;
    use rand::Rng;

    #[test]
    fn test_loop_behavior() {
        {
            let mut rng = rand::thread_rng();
            let v = LoopBehavior::Finite(NonZeroU32::new(rng.gen_range(1..=0xFFFFFFFF)).unwrap());
            let msg = v.to_msg(None);
            let v2 = LoopBehavior::from_msg(&msg).unwrap();
            assert_eq!(v, v2);
        }

        {
            let v = LoopBehavior::Infinite;
            let msg = v.to_msg(None);
            let v2 = LoopBehavior::from_msg(&msg).unwrap();
            assert_eq!(v, v2);
        }
    }
}
