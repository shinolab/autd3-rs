use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<autd3_driver::firmware::fpga::Phase> for Phase {
    fn from(value: autd3_driver::firmware::fpga::Phase) -> Self {
        Self {
            value: value.0 as _,
        }
    }
}

impl FromMessage<Phase> for autd3_driver::firmware::fpga::Phase {
    fn from_msg(msg: Phase) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::firmware::fpga::Phase(u8::try_from(
            msg.value,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::Phase;
    use rand::Rng;

    #[test]
    fn test_phase() {
        let mut rng = rand::rng();
        let v = Phase(rng.random());
        let msg = v.into();
        let v2 = Phase::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }
}
