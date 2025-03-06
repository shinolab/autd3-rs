use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<autd3_gain_holo::Amplitude> for Amplitude {
    fn from(value: autd3_gain_holo::Amplitude) -> Self {
        Self {
            value: value.pascal(),
        }
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
        let msg = v.into();
        let v2 = autd3_gain_holo::Amplitude::from_msg(msg).unwrap();
        approx::assert_abs_diff_eq!(v.pascal(), v2.pascal());
    }
}
