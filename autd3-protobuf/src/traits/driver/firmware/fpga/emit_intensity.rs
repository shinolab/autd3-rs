use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::firmware::fpga::EmitIntensity {
    type Message = EmitIntensity;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            value: self.value() as _,
        }
    }
}

impl FromMessage<EmitIntensity> for autd3_driver::firmware::fpga::EmitIntensity {
    fn from_msg(msg: &EmitIntensity) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::firmware::fpga::EmitIntensity::new(
            msg.value as _,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::derive::EmitIntensity;
    use rand::Rng;

    #[test]
    fn test_emit_intensity() {
        let mut rng = rand::thread_rng();
        let v = EmitIntensity::new(rng.gen());
        let msg = v.to_msg(None);
        let v2 = EmitIntensity::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
