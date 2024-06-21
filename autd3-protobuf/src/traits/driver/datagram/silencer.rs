use std::time::Duration;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::datagram::SilencerFixedUpdateRate {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedUpdateRate(SilencerFixedUpdateRate {
                    value_intensity: self.update_rate_intensity() as _,
                    value_phase: self.update_rate_phase() as _,
                })),
            })),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<SilencerFixedUpdateRate> for autd3_driver::datagram::SilencerFixedUpdateRate {
    fn from_msg(msg: &SilencerFixedUpdateRate) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Silencer::from_update_rate(
            msg.value_intensity as _,
            msg.value_phase as _,
        ))
    }
}

impl ToMessage for autd3_driver::datagram::SilencerFixedCompletionSteps {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedCompletionSteps(
                    SilencerFixedCompletionSteps {
                        value_intensity: self.completion_steps_intensity() as _,
                        value_phase: self.completion_steps_phase() as _,
                        strict_mode: Some(self.strict_mode()),
                    },
                )),
            })),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<SilencerFixedCompletionSteps>
    for autd3_driver::datagram::SilencerFixedCompletionSteps
{
    fn from_msg(msg: &SilencerFixedCompletionSteps) -> Result<Self, AUTDProtoBufError> {
        let mut s = autd3_driver::datagram::Silencer::from_completion_steps(
            msg.value_intensity as _,
            msg.value_phase as _,
        );
        if let Some(strict_mode) = msg.strict_mode {
            s = s.with_strict_mode(strict_mode);
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
                        value_intensity: self.completion_time_intensity().as_nanos() as _,
                        value_phase: self.completion_time_phase().as_nanos() as _,
                        strict_mode: Some(self.strict_mode()),
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
            Duration::from_nanos(msg.value_intensity as _),
            Duration::from_nanos(msg.value_phase as _),
        );
        if let Some(strict_mode) = msg.strict_mode {
            s = s.with_strict_mode(strict_mode);
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::{SILENCER_VALUE_MAX, SILENCER_VALUE_MIN};
    use rand::Rng;

    #[test]
    fn test_silencer_fixed_update_rate() {
        let mut rng = rand::thread_rng();

        let c = autd3_driver::datagram::Silencer::from_update_rate(
            rng.gen_range(SILENCER_VALUE_MIN..SILENCER_VALUE_MAX),
            rng.gen_range(SILENCER_VALUE_MIN..SILENCER_VALUE_MAX),
        );
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

    #[test]
    fn test_silencer_fixed_completion_steps() {
        let mut rng = rand::thread_rng();

        let c = autd3_driver::datagram::Silencer::from_completion_steps(
            rng.gen_range(SILENCER_VALUE_MIN..SILENCER_VALUE_MAX),
            rng.gen_range(SILENCER_VALUE_MIN..SILENCER_VALUE_MAX),
        )
        .with_strict_mode(false);
        let msg = c.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedCompletionSteps(config)),
            })) => {
                let c2 = autd3_driver::datagram::SilencerFixedCompletionSteps::from_msg(&config)
                    .unwrap();
                assert_eq!(
                    c2.completion_steps_intensity(),
                    c.completion_steps_intensity()
                );
                assert_eq!(c2.completion_steps_phase(), c.completion_steps_phase());
                assert_eq!(c2.strict_mode(), c.strict_mode());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
