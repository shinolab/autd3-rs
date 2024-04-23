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
