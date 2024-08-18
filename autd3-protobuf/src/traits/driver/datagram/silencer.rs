use std::{num::NonZeroU16, time::Duration};

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

fn silencer_target_to(v: i32) -> Result<autd3::prelude::SilencerTarget, AUTDProtoBufError> {
    if v == autd3::prelude::SilencerTarget::Intensity as u8 as _ {
        Ok(autd3::prelude::SilencerTarget::Intensity)
    } else if v == autd3::prelude::SilencerTarget::PulseWidth as u8 as _ {
        Ok(autd3::prelude::SilencerTarget::PulseWidth)
    } else {
        Err(AUTDProtoBufError::DataParseError)
    }
}

impl ToMessage for autd3_driver::datagram::SilencerFixedUpdateRate {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedUpdateRate(SilencerFixedUpdateRate {
                    value_intensity: self.update_rate_intensity().get() as _,
                    value_phase: self.update_rate_phase().get() as _,
                    target: Some(self.target() as u8 as _),
                })),
            })),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<SilencerFixedUpdateRate> for autd3_driver::datagram::SilencerFixedUpdateRate {
    fn from_msg(msg: &SilencerFixedUpdateRate) -> Result<Self, AUTDProtoBufError> {
        let mut s = autd3_driver::datagram::Silencer::from_update_rate(
            NonZeroU16::new(msg.value_intensity as _).ok_or(AUTDProtoBufError::DataParseError)?,
            NonZeroU16::new(msg.value_phase as _).ok_or(AUTDProtoBufError::DataParseError)?,
        );
        if let Some(target) = msg.target {
            s = s.with_target(silencer_target_to(target)?);
        }
        Ok(s)
    }
}

impl ToMessage for autd3_driver::datagram::SilencerFixedCompletionTime {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedCompletionTime(
                    SilencerFixedCompletionTime {
                        value_intensity: self.completion_time_intensity().as_micros() as _,
                        value_phase: self.completion_time_phase().as_micros() as _,
                        strict_mode: Some(self.strict_mode()),
                        target: Some(self.target() as u8 as _),
                    },
                )),
            })),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<SilencerFixedCompletionTime>
    for autd3_driver::datagram::SilencerFixedCompletionTime
{
    fn from_msg(msg: &SilencerFixedCompletionTime) -> Result<Self, AUTDProtoBufError> {
        let mut s = autd3_driver::datagram::Silencer::from_completion_time(
            Duration::from_micros(msg.value_intensity as _),
            Duration::from_micros(msg.value_phase as _),
        );
        if let Some(strict_mode) = msg.strict_mode {
            s = s.with_strict_mode(strict_mode);
        }
        if let Some(target) = msg.target {
            s = s.with_target(silencer_target_to(target)?);
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_silencer_fixed_update_rate() {
        let mut rng = rand::thread_rng();

        let c = unsafe {
            autd3_driver::datagram::Silencer::from_update_rate(
                NonZeroU16::new_unchecked(rng.gen_range(1..=u16::MAX)),
                NonZeroU16::new_unchecked(rng.gen_range(1..=u16::MAX)),
            )
        };
        let msg = c.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedUpdateRate(config)),
            })) => {
                let c2 =
                    autd3_driver::datagram::SilencerFixedUpdateRate::from_msg(&config).unwrap();
                assert_eq!(c2.update_rate_intensity(), c.update_rate_intensity());
                assert_eq!(c2.update_rate_phase(), c.update_rate_phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
