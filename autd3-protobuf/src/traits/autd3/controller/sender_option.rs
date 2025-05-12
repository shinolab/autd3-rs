use std::time::Duration;

use crate::{AUTDProtoBufError, FromMessage, ParallelMode, SenderOption};

impl From<ParallelMode> for autd3::controller::ParallelMode {
    fn from(value: ParallelMode) -> Self {
        match value {
            ParallelMode::Auto => Self::Auto,
            ParallelMode::On => Self::On,
            ParallelMode::Off => Self::Off,
        }
    }
}

impl From<autd3::controller::ParallelMode> for ParallelMode {
    fn from(value: autd3::controller::ParallelMode) -> Self {
        match value {
            autd3::prelude::ParallelMode::Auto => ParallelMode::Auto,
            autd3::prelude::ParallelMode::On => ParallelMode::On,
            autd3::prelude::ParallelMode::Off => ParallelMode::Off,
        }
    }
}

impl FromMessage<i32> for autd3::controller::ParallelMode {
    fn from_msg(msg: i32) -> Result<Self, AUTDProtoBufError> {
        Ok(ParallelMode::try_from(msg)?.into())
    }
}

impl From<autd3::controller::SenderOption> for SenderOption {
    fn from(value: autd3::controller::SenderOption) -> Self {
        SenderOption {
            send_interval_ns: value.send_interval.as_nanos() as _,
            receive_interval_ns: value.receive_interval.as_nanos() as _,
            timeout_ns: value.timeout.map(|t| t.as_nanos() as _),
            parallel: ParallelMode::from(value.parallel) as _,
        }
    }
}

impl FromMessage<SenderOption> for autd3::controller::SenderOption {
    fn from_msg(msg: SenderOption) -> Result<Self, AUTDProtoBufError> {
        let send_interval = Duration::from_nanos(msg.send_interval_ns);
        let receive_interval = Duration::from_nanos(msg.receive_interval_ns);
        let timeout = msg.timeout_ns.map(Duration::from_nanos);
        let parallel = autd3::controller::ParallelMode::from_msg(msg.parallel)?;
        Ok(autd3::controller::SenderOption {
            send_interval,
            receive_interval,
            timeout,
            parallel,
        })
    }
}
