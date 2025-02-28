use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_gain_holo::Amplitude {
    type Message = Amplitude;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            value: self.pascal() as _,
        })
    }
}

impl FromMessage<Amplitude> for autd3_gain_holo::Amplitude {
    fn from_msg(msg: Amplitude) -> Result<Self, AUTDProtoBufError> {
        Ok(msg.value * autd3_gain_holo::Pa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_gain_holo::Pa;
    use rand::Rng;

    #[test]
    fn test_amp() {
        let mut rng = rand::rng();
        let v = rng.random::<f32>() * Pa;
        let msg = v.to_msg(None).unwrap();
        let v2 = autd3_gain_holo::Amplitude::from_msg(msg).unwrap();
        approx::assert_abs_diff_eq!(v.pascal(), v2.pascal());
    }
}
