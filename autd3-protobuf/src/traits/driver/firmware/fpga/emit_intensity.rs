use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<autd3_driver::firmware::fpga::EmitIntensity> for EmitIntensity {
    fn from(value: autd3_driver::firmware::fpga::EmitIntensity) -> Self {
        Self {
            value: value.0 as _,
        }
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
        let msg = v.into();
        let v2 = EmitIntensity::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }
}
