use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::firmware::fpga::EmitIntensity {
    type Message = EmitIntensity;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message { value: self.0 as _ })
    }
}

impl FromMessage<EmitIntensity> for autd3_driver::firmware::fpga::EmitIntensity {
    fn from_msg(msg: EmitIntensity) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::firmware::fpga::EmitIntensity(u8::try_from(
            msg.value,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::EmitIntensity;
    use rand::Rng;

    #[test]
    fn test_emit_intensity() {
        let mut rng = rand::rng();
        let v = EmitIntensity(rng.random());
        let msg = v.to_msg(None).unwrap();
        let v2 = EmitIntensity::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }
}
