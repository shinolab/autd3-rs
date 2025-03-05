use crate::{
    AUTDProtoBufError, AsyncSleeper, FromMessage, ParallelMode, SenderOption, SpinSleeper,
    SpinStrategy, StdSleeper, ToMessage, WaitableSleeper,
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

impl ToMessage for autd3::controller::ParallelMode {
    type Message = i32;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(ParallelMode::from(*self) as _)
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

impl ToMessage for autd3::controller::SpinStrategy {
    type Message = i32;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(SpinStrategy::from(*self) as _)
    }
}

impl FromMessage<i32> for autd3::controller::SpinStrategy {
    fn from_msg(msg: i32) -> Result<Self, AUTDProtoBufError> {
        Ok(SpinStrategy::try_from(msg)?.into())
    }
}

impl ToMessage for autd3::controller::SenderOption<autd3::controller::StdSleeper> {
    type Message = SenderOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(SenderOption {
            send_interval_ns: self.send_interval.as_nanos() as _,
            receive_interval_ns: self.receive_interval.as_nanos() as _,
            timeout_ns: self.timeout.map(|t| t.as_nanos() as _),
            parallel: self.parallel.to_msg(None)?,
            sleeper: Some(crate::sender_option::Sleeper::Std(StdSleeper {
                timer_resolution: self.sleeper.timer_resolution.map(|t| t.get()),
            })),
        })
    }
}

impl ToMessage for autd3::controller::SenderOption<autd3::controller::SpinSleeper> {
    type Message = SenderOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(SenderOption {
            send_interval_ns: self.send_interval.as_nanos() as _,
            receive_interval_ns: self.receive_interval.as_nanos() as _,
            timeout_ns: self.timeout.map(|t| t.as_nanos() as _),
            parallel: self.parallel.to_msg(None)?,
            sleeper: Some(crate::sender_option::Sleeper::Spin(SpinSleeper {
                native_accuracy_ns: self.sleeper.native_accuracy_ns(),
                spin_strategy: self.sleeper.spin_strategy().to_msg(None)?,
            })),
        })
    }
}

#[cfg(target_os = "windows")]
impl ToMessage for autd3::controller::SenderOption<autd3::controller::WaitableSleeper> {
    type Message = SenderOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(SenderOption {
            send_interval_ns: self.send_interval.as_nanos() as _,
            receive_interval_ns: self.receive_interval.as_nanos() as _,
            timeout_ns: self.timeout.map(|t| t.as_nanos() as _),
            parallel: self.parallel.to_msg(None)?,
            sleeper: Some(crate::sender_option::Sleeper::Waitable(WaitableSleeper {})),
        })
    }
}

impl ToMessage for autd3::controller::SenderOption<autd3::r#async::controller::AsyncSleeper> {
    type Message = SenderOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(SenderOption {
            send_interval_ns: self.send_interval.as_nanos() as _,
            receive_interval_ns: self.receive_interval.as_nanos() as _,
            timeout_ns: self.timeout.map(|t| t.as_nanos() as _),
            parallel: self.parallel.to_msg(None)?,
            sleeper: Some(crate::sender_option::Sleeper::Async(AsyncSleeper {
                timer_resolution: self.sleeper.timer_resolution.map(|t| t.get()),
            })),
        })
    }
}
