use std::{num::NonZeroU16, time::Duration};

use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedUpdateRate> {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedUpdateRate(
                    silencer::FixedUpdateRate {
                        value_intensity: self.config.intensity.get() as _,
                        value_phase: self.config.phase.get() as _,
                    },
                )),
                target: self.target as u8 as _,
            })),
        })
    }
}

impl ToMessage for autd3_driver::datagram::Silencer<autd3_driver::datagram::FixedCompletionSteps> {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Silencer(Silencer {
                config: Some(silencer::Config::FixedCompletionSteps(
                    silencer::FixedCompletionSteps {
                        value_intensity: Some(self.config.intensity.get() as _),
                        value_phase: Some(self.config.phase.get() as _),
                        strict_mode: Some(self.config.strict_mode),
                    },
                )),
                target: self.target as u8 as _,
            })),
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
                    silencer::FixedCompletionTime {
                        value_intensity: Some(self.config.intensity.as_micros() as _),
                        value_phase: Some(self.config.phase.as_micros() as _),
                        strict_mode: Some(self.config.strict_mode),
                    },
                )),
                target: self.target as u8 as _,
            })),
        })
    }
}

impl From<SilencerTarget> for autd3::driver::firmware::fpga::SilencerTarget {
    fn from(value: SilencerTarget) -> Self {
        match value {
            SilencerTarget::Intensity => autd3::driver::firmware::fpga::SilencerTarget::Intensity,
            SilencerTarget::PulseWidth => autd3::driver::firmware::fpga::SilencerTarget::PulseWidth,
        }
    }
}

impl From<autd3::driver::firmware::fpga::SilencerTarget> for SilencerTarget {
    fn from(value: autd3::driver::firmware::fpga::SilencerTarget) -> Self {
        match value {
            autd3::driver::firmware::fpga::SilencerTarget::Intensity => SilencerTarget::Intensity,
            autd3::driver::firmware::fpga::SilencerTarget::PulseWidth => SilencerTarget::PulseWidth,
        }
    }
}

impl FromMessage<i32> for autd3::driver::firmware::fpga::SilencerTarget {
    fn from_msg(msg: i32) -> Result<Self, AUTDProtoBufError> {
        Ok(SilencerTarget::try_from(msg)?.into())
    }
}

impl FromMessage<silencer::FixedUpdateRate> for autd3_driver::datagram::FixedUpdateRate {
    fn from_msg(msg: silencer::FixedUpdateRate) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::FixedUpdateRate {
            intensity: NonZeroU16::new(msg.value_intensity as _)
                .ok_or(AUTDProtoBufError::DataParseError)?,
            phase: NonZeroU16::new(msg.value_phase as _)
                .ok_or(AUTDProtoBufError::DataParseError)?,
        })
    }
}

impl FromMessage<silencer::FixedCompletionTime> for autd3_driver::datagram::FixedCompletionTime {
    fn from_msg(msg: silencer::FixedCompletionTime) -> Result<Self, AUTDProtoBufError> {
        let default = autd3_driver::datagram::FixedCompletionTime::default();
        Ok(autd3_driver::datagram::FixedCompletionTime {
            intensity: msg
                .value_intensity
                .map(|v| Duration::from_micros(v as _))
                .unwrap_or(default.intensity),
            phase: msg
                .value_phase
                .map(|v| Duration::from_micros(v as _))
                .unwrap_or(default.phase),
            strict_mode: msg.strict_mode.unwrap_or(default.strict_mode),
        })
    }
}

impl FromMessage<silencer::FixedCompletionSteps> for autd3_driver::datagram::FixedCompletionSteps {
    fn from_msg(msg: silencer::FixedCompletionSteps) -> Result<Self, AUTDProtoBufError> {
        let default = autd3_driver::datagram::FixedCompletionSteps::default();
        Ok(autd3_driver::datagram::FixedCompletionSteps {
            intensity: msg
                .value_intensity
                .map(u16::try_from)
                .transpose()?
                .map(NonZeroU16::try_from)
                .transpose()?
                .unwrap_or(default.intensity),
            phase: msg
                .value_phase
                .map(u16::try_from)
                .transpose()?
                .map(NonZeroU16::try_from)
                .transpose()?
                .unwrap_or(default.phase),
            strict_mode: msg.strict_mode.unwrap_or(default.strict_mode),
        })
    }
}
