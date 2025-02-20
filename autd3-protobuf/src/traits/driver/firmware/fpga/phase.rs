use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::firmware::fpga::Phase {
    type Message = Phase;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message { value: self.0 as _ })
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
        let msg = v.to_msg(None).unwrap();
        let v2 = Phase::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }
}
