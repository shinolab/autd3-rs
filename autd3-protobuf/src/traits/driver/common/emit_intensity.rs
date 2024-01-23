use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::common::EmitIntensity {
    type Message = EmitIntensity;

    fn to_msg(&self) -> Self::Message {
        Self::Message {
            value: self.value() as _,
        }
    }
}

impl FromMessage<EmitIntensity> for autd3_driver::common::EmitIntensity {
    fn from_msg(msg: &EmitIntensity) -> Option<Self> {
        Some(autd3_driver::common::EmitIntensity::new(msg.value as _))
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
        let msg = v.to_msg();
        let v2 = EmitIntensity::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
