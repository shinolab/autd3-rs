use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::firmware::fpga::Phase {
    type Message = Phase;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            value: self.value() as _,
        }
    }
}

impl FromMessage<Phase> for autd3_driver::firmware::fpga::Phase {
    fn from_msg(msg: &Phase) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::firmware::fpga::Phase::new(msg.value as _))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::Phase;
    use rand::Rng;

    #[test]
    fn test_phase() {
        let mut rng = rand::thread_rng();
        let v = Phase::new(rng.gen());
        let msg = v.to_msg(None);
        let v2 = Phase::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
