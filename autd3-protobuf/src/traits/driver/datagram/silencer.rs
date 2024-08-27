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

impl ToMessage for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedUpdateRate> {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedUpdateRate(SilencerFixedUpdateRate {
                    value_intensity: self.config().intensity().get() as _,
                    value_phase: self.config().phase().get() as _,
                    target: Some(self.target() as u8 as _),
                })),
            })),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<SilencerFixedUpdateRate>
    for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedUpdateRate>
{
    fn from_msg(msg: &SilencerFixedUpdateRate) -> Result<Self, AUTDProtoBufError> {
        let mut s =
            autd3_driver::datagram::Silencer::new(autd3_driver::datagram::FixedUpdateRate {
                intensity: NonZeroU16::new(msg.value_intensity as _)
                    .ok_or(AUTDProtoBufError::DataParseError)?,
                phase: NonZeroU16::new(msg.value_phase as _)
                    .ok_or(AUTDProtoBufError::DataParseError)?,
            });
        if let Some(target) = msg.target {
            s = s.with_target(silencer_target_to(target)?);
        }
        Ok(s)
    }
}

impl ToMessage for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedCompletionTime> {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedCompletionTime(
                    SilencerFixedCompletionTime {
                        value_intensity: self.config().intensity().as_micros() as _,
                        value_phase: self.config().phase().as_micros() as _,
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
    for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedCompletionTime>
{
    fn from_msg(msg: &SilencerFixedCompletionTime) -> Result<Self, AUTDProtoBufError> {
        let mut s =
            autd3_driver::datagram::Silencer::new(autd3_driver::datagram::FixedCompletionTime {
                intensity: Duration::from_micros(msg.value_intensity as _),
                phase: Duration::from_micros(msg.value_phase as _),
            });
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

        let c = autd3_driver::datagram::Silencer::new(autd3_driver::datagram::FixedUpdateRate {
            intensity: NonZeroU16::new(rng.gen_range(1..=u16::MAX)).unwrap(),
            phase: NonZeroU16::new(rng.gen_range(1..=u16::MAX)).unwrap(),
        });
        let msg = c.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedUpdateRate(config)),
            })) => {
                let c2 =
                    autd3_driver::datagram::Silencer::<autd3_driver::datagram::FixedUpdateRate>::from_msg(&config).unwrap();
                assert_eq!(c2.config().intensity(), c.config().intensity());
                assert_eq!(c2.config().phase(), c.config().phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
