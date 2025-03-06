use std::num::NonZeroU16;

use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<autd3_driver::firmware::fpga::LoopBehavior> for LoopBehavior {
    fn from(value: autd3_driver::firmware::fpga::LoopBehavior) -> Self {
        match value {
            autd3::prelude::LoopBehavior::Infinite => LoopBehavior {
                variant: Some(loop_behavior::Variant::Infinite(loop_behavior::Infinite {})),
            },
            autd3::prelude::LoopBehavior::Finite(rep) => LoopBehavior {
                variant: Some(loop_behavior::Variant::Finite(loop_behavior::Finite {
                    rep: rep.get() as _,
                })),
            },
        }
    }
}

impl FromMessage<LoopBehavior> for autd3_driver::firmware::fpga::LoopBehavior {
    fn from_msg(msg: LoopBehavior) -> Result<Self, AUTDProtoBufError> {
        Ok(
            match msg.variant.ok_or(AUTDProtoBufError::DataParseError)? {
                loop_behavior::Variant::Infinite(_) => {
                    autd3_driver::firmware::fpga::LoopBehavior::Infinite
                }
                loop_behavior::Variant::Finite(value) => {
                    autd3_driver::firmware::fpga::LoopBehavior::Finite(NonZeroU16::try_from(
                        u16::try_from(value.rep)?,
                    )?)
                }
            },
        )
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
            let mut rng = rand::rng();
            let v = LoopBehavior::Finite(NonZeroU16::new(rng.random_range(1..=0xFFFF)).unwrap());
            let msg = v.into();
            let v2 = LoopBehavior::from_msg(msg).unwrap();
            assert_eq!(v, v2);
        }

        {
            let v = LoopBehavior::Infinite;
            let msg = v.into();
            let v2 = LoopBehavior::from_msg(msg).unwrap();
            assert_eq!(v, v2);
        }
    }
}
