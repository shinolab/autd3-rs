use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::ConfigureSilencerFixedUpdateRate {
    type Message = DatagramLightweight;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Silencer(
                ConfigureSilencer {
                    config: Some(configure_silencer::Config::FixedUpdateRate(
                        ConfigureSilencerFixedUpdateRate {
                            value_intensity: self.update_rate_intensity() as _,
                            value_phase: self.update_rate_phase() as _,
                        },
                    )),
                },
            )),
        }
    }
}

impl FromMessage<ConfigureSilencerFixedUpdateRate>
    for autd3_driver::datagram::ConfigureSilencerFixedUpdateRate
{
    fn from_msg(msg: &ConfigureSilencerFixedUpdateRate) -> Option<Self> {
        autd3_driver::datagram::ConfigureSilencer::fixed_update_rate(
            msg.value_intensity as _,
            msg.value_phase as _,
        )
        .ok()
    }
}

impl ToMessage for autd3_driver::datagram::ConfigureSilencerFixedCompletionSteps {
    type Message = DatagramLightweight;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Silencer(
                ConfigureSilencer {
                    config: Some(configure_silencer::Config::FixedCompletionSteps(
                        ConfigureSilencerFixedCompletionSteps {
                            value_intensity: self.completion_steps_intensity() as _,
                            value_phase: self.completion_steps_phase() as _,
                            strict_mode: self.strict_mode(),
                        },
                    )),
                },
            )),
        }
    }
}

impl FromMessage<ConfigureSilencerFixedCompletionSteps>
    for autd3_driver::datagram::ConfigureSilencerFixedCompletionSteps
{
    fn from_msg(msg: &ConfigureSilencerFixedCompletionSteps) -> Option<Self> {
        Some(
            autd3_driver::datagram::ConfigureSilencer::fixed_completion_steps(
                msg.value_intensity as _,
                msg.value_phase as _,
            )
            .ok()?
            .with_strict_mode(msg.strict_mode),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::fpga::{SILENCER_VALUE_MAX, SILENCER_VALUE_MIN};
    use rand::Rng;

    #[test]
    fn test_silencer_fixed_update_rate() {
        let mut rng = rand::thread_rng();

        let c = autd3_driver::datagram::ConfigureSilencer::fixed_update_rate(
            rng.gen_range(SILENCER_VALUE_MIN..SILENCER_VALUE_MAX),
            rng.gen_range(SILENCER_VALUE_MIN..SILENCER_VALUE_MAX),
        )
        .unwrap();
        let msg = c.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Silencer(ConfigureSilencer {
                config: Some(configure_silencer::Config::FixedUpdateRate(config)),
            })) => {
                let c2 =
                    autd3_driver::datagram::ConfigureSilencerFixedUpdateRate::from_msg(&config)
                        .unwrap();
                assert_eq!(c2.update_rate_intensity(), c.update_rate_intensity());
                assert_eq!(c2.update_rate_phase(), c.update_rate_phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }

    #[test]
    fn test_silencer_fixed_completion_steps() {
        let mut rng = rand::thread_rng();

        let c = autd3_driver::datagram::ConfigureSilencer::fixed_completion_steps(
            rng.gen_range(SILENCER_VALUE_MIN..SILENCER_VALUE_MAX),
            rng.gen_range(SILENCER_VALUE_MIN..SILENCER_VALUE_MAX),
        )
        .unwrap()
        .with_strict_mode(false);
        let msg = c.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Silencer(ConfigureSilencer {
                config: Some(configure_silencer::Config::FixedCompletionSteps(config)),
            })) => {
                let c2 = autd3_driver::datagram::ConfigureSilencerFixedCompletionSteps::from_msg(
                    &config,
                )
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
