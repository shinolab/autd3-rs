use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::firmware::fpga::LoopBehavior {
    type Message = LoopBehavior;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            rep: self.rep() as _,
        }
    }
}

impl FromMessage<LoopBehavior> for autd3_driver::firmware::fpga::LoopBehavior {
    fn from_msg(msg: &LoopBehavior) -> Result<Self, AUTDProtoBufError> {
        Ok(match msg.rep {
            0xFFFF => autd3_driver::firmware::fpga::LoopBehavior::infinite(),
            v => autd3_driver::firmware::fpga::LoopBehavior::finite(v as u16 + 1).unwrap(),
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
            let v = LoopBehavior::finite(rng.gen_range(1..=0xFFFF)).unwrap();
            let msg = v.to_msg(None);
            let v2 = LoopBehavior::from_msg(&msg).unwrap();
            assert_eq!(v, v2);
        }

        {
            let v = LoopBehavior::infinite();
            let msg = v.to_msg(None);
            let v2 = LoopBehavior::from_msg(&msg).unwrap();
            assert_eq!(v, v2);
        }
    }
}
