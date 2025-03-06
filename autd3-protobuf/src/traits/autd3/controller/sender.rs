use crate::{
    AUTDProtoBufError, AsyncSleeper, FromMessage, ParallelMode, SenderOption, SpinSleeper,
    SpinStrategy, StdSleeper,
};

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

impl From<SpinStrategy> for autd3::controller::SpinStrategy {
    fn from(value: SpinStrategy) -> Self {
        match value {
            SpinStrategy::SpinLoopHint => Self::SpinLoopHint,
            SpinStrategy::YieldThread => Self::YieldThread,
        }
    }
}

impl From<autd3::controller::SpinStrategy> for SpinStrategy {
    fn from(value: autd3::controller::SpinStrategy) -> Self {
        match value {
            autd3::controller::SpinStrategy::SpinLoopHint => SpinStrategy::SpinLoopHint,
            autd3::controller::SpinStrategy::YieldThread => SpinStrategy::YieldThread,
            _ => unimplemented!(),
        }
    }
}

impl FromMessage<i32> for autd3::controller::SpinStrategy {
    fn from_msg(msg: i32) -> Result<Self, AUTDProtoBufError> {
        Ok(SpinStrategy::try_from(msg)?.into())
    }
}

impl From<&autd3::controller::SenderOption<autd3::controller::StdSleeper>> for SenderOption {
    fn from(value: &autd3::controller::SenderOption<autd3::controller::StdSleeper>) -> Self {
        SenderOption {
            send_interval_ns: value.send_interval.as_nanos() as _,
            receive_interval_ns: value.receive_interval.as_nanos() as _,
            timeout_ns: value.timeout.map(|t| t.as_nanos() as _),
            parallel: ParallelMode::from(value.parallel) as _,
            sleeper: Some(crate::sender_option::Sleeper::Std(StdSleeper {
                timer_resolution: value.sleeper.timer_resolution.map(|t| t.get()),
            })),
        }
    }
}

impl From<&autd3::controller::SenderOption<autd3::controller::SpinSleeper>> for SenderOption {
    fn from(value: &autd3::controller::SenderOption<autd3::controller::SpinSleeper>) -> Self {
        SenderOption {
            send_interval_ns: value.send_interval.as_nanos() as _,
            receive_interval_ns: value.receive_interval.as_nanos() as _,
            timeout_ns: value.timeout.map(|t| t.as_nanos() as _),
            parallel: ParallelMode::from(value.parallel) as _,
            sleeper: Some(crate::sender_option::Sleeper::Spin(SpinSleeper {
                native_accuracy_ns: value.sleeper.native_accuracy_ns(),
                spin_strategy: SpinStrategy::from(value.sleeper.spin_strategy()) as _,
            })),
        }
    }
}

#[cfg(target_os = "windows")]
impl From<&autd3::controller::SenderOption<autd3::controller::WaitableSleeper>> for SenderOption {
    fn from(value: &autd3::controller::SenderOption<autd3::controller::WaitableSleeper>) -> Self {
        SenderOption {
            send_interval_ns: value.send_interval.as_nanos() as _,
            receive_interval_ns: value.receive_interval.as_nanos() as _,
            timeout_ns: value.timeout.map(|t| t.as_nanos() as _),
            parallel: ParallelMode::from(value.parallel) as _,
            sleeper: Some(crate::sender_option::Sleeper::Waitable(
                crate::WaitableSleeper {},
            )),
        }
    }
}

impl From<&autd3::controller::SenderOption<autd3::r#async::controller::AsyncSleeper>>
    for SenderOption
{
    fn from(
        value: &autd3::controller::SenderOption<autd3::r#async::controller::AsyncSleeper>,
    ) -> Self {
        SenderOption {
            send_interval_ns: value.send_interval.as_nanos() as _,
            receive_interval_ns: value.receive_interval.as_nanos() as _,
            timeout_ns: value.timeout.map(|t| t.as_nanos() as _),
            parallel: ParallelMode::from(value.parallel) as _,
            sleeper: Some(crate::sender_option::Sleeper::Async(AsyncSleeper {
                timer_resolution: value.sleeper.timer_resolution.map(|t| t.get()),
            })),
        }
    }
}
