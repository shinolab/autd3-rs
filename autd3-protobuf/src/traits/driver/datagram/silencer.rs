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

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedUpdateRate(SilencerFixedUpdateRate {
                    value_intensity: self.config.intensity.get() as _,
                    value_phase: self.config.phase.get() as _,
                    target: Some(self.target as u8 as _),
                })),
            })),
        })
    }
}

impl FromMessage<SilencerFixedUpdateRate>
    for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedUpdateRate>
{
    fn from_msg(msg: &SilencerFixedUpdateRate) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Silencer {
            config: autd3_driver::datagram::FixedUpdateRate {
                intensity: NonZeroU16::new(msg.value_intensity as _)
                    .ok_or(AUTDProtoBufError::DataParseError)?,
                phase: NonZeroU16::new(msg.value_phase as _)
                    .ok_or(AUTDProtoBufError::DataParseError)?,
            },
            target: msg
                .target
                .as_ref()
                .map(|v| silencer_target_to(*v))
                .transpose()?
                .ok_or(AUTDProtoBufError::DataParseError)?,
        })
    }
}

impl ToMessage for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedCompletionTime> {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedCompletionTime(
                    SilencerFixedCompletionTime {
                        value_intensity: Some(self.config.intensity.as_micros() as _),
                        value_phase: Some(self.config.phase.as_micros() as _),
                        strict_mode: Some(self.config.strict_mode),
                        target: Some(self.target as u8 as _),
                    },
                )),
            })),
        })
    }
}

impl FromMessage<SilencerFixedCompletionTime>
    for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedCompletionTime>
{
    fn from_msg(msg: &SilencerFixedCompletionTime) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Silencer {
            config: autd3_driver::datagram::FixedCompletionTime {
                intensity: msg
                    .value_intensity
                    .map(|v| Duration::from_micros(v as _))
                    .unwrap_or(autd3_driver::datagram::FixedCompletionTime::default().intensity),
                phase: msg
                    .value_phase
                    .map(|v| Duration::from_micros(v as _))
                    .unwrap_or(autd3_driver::datagram::FixedCompletionTime::default().phase),
                strict_mode: msg
                    .strict_mode
                    .unwrap_or(autd3_driver::datagram::FixedCompletionTime::default().strict_mode),
            },
            target: msg
                .target
                .as_ref()
                .map(|v| silencer_target_to(*v))
                .transpose()?
                .ok_or(AUTDProtoBufError::DataParseError)?,
        })
    }
}

impl FromMessage<SilencerFixedCompletionSteps>
    for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedCompletionSteps>
{
    fn from_msg(msg: &SilencerFixedCompletionSteps) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Silencer {
            config: autd3_driver::datagram::FixedCompletionSteps {
                intensity: msg
                    .value_intensity
                    .map(u16::try_from)
                    .transpose()?
                    .map(NonZeroU16::try_from)
                    .transpose()?
                    .unwrap_or(autd3_driver::datagram::FixedCompletionSteps::default().intensity),
                phase: msg
                    .value_phase
                    .map(u16::try_from)
                    .transpose()?
                    .map(NonZeroU16::try_from)
                    .transpose()?
                    .unwrap_or(autd3_driver::datagram::FixedCompletionSteps::default().phase),
                strict_mode: msg
                    .strict_mode
                    .unwrap_or(autd3_driver::datagram::FixedCompletionSteps::default().strict_mode),
            },
            target: msg
                .target
                .as_ref()
                .map(|v| silencer_target_to(*v))
                .transpose()?
                .unwrap_or(Self::default().target),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_silencer_fixed_update_rate() {
        let mut rng = rand::thread_rng();

        let c = autd3_driver::datagram::Silencer {
            config: autd3_driver::datagram::FixedUpdateRate {
                intensity: NonZeroU16::new(rng.gen_range(1..=u16::MAX)).unwrap(),
                phase: NonZeroU16::new(rng.gen_range(1..=u16::MAX)).unwrap(),
            },
            target: autd3::prelude::SilencerTarget::Intensity,
        };
        let msg = c.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedUpdateRate(config)),
            })) => {
                let c2 =
                    autd3_driver::datagram::Silencer::<autd3_driver::datagram::FixedUpdateRate>::from_msg(&config).unwrap();
                assert_eq!(c2.config.intensity, c.config.intensity);
                assert_eq!(c2.config.phase, c.config.phase);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
