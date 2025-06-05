use crate::{AUTDProtoBufError, FromMessage, Sleeper, SpinStrategy, sleeper};

impl From<SpinStrategy> for autd3_core::sleep::SpinStrategy {
    fn from(value: SpinStrategy) -> Self {
        match value {
            SpinStrategy::SpinLoopHint => Self::SpinLoopHint,
            SpinStrategy::YieldThread => Self::YieldThread,
        }
    }
}

impl From<autd3_core::sleep::SpinStrategy> for SpinStrategy {
    fn from(value: autd3_core::sleep::SpinStrategy) -> Self {
        match value {
            autd3_core::sleep::SpinStrategy::SpinLoopHint => SpinStrategy::SpinLoopHint,
            autd3_core::sleep::SpinStrategy::YieldThread => SpinStrategy::YieldThread,
            _ => unimplemented!(),
        }
    }
}

impl FromMessage<i32> for autd3_core::sleep::SpinStrategy {
    fn from_msg(msg: i32) -> Result<Self, AUTDProtoBufError> {
        Ok(SpinStrategy::try_from(msg)?.into())
    }
}

impl FromMessage<Sleeper> for Box<dyn autd3::r#async::controller::AsyncSleep + Send + Sync> {
    fn from_msg(value: Sleeper) -> Result<Self, AUTDProtoBufError> {
        Ok(match value.sleeper {
            Some(sleeper::Sleeper::Std(_)) => Box::new(autd3_core::sleep::StdSleeper),
            Some(sleeper::Sleeper::Spin(spin_sleeper)) => Box::new(
                autd3_core::sleep::SpinSleeper::new(spin_sleeper.native_accuracy_ns)
                    .with_spin_strategy(autd3_core::sleep::SpinStrategy::from_msg(
                        spin_sleeper.spin_strategy,
                    )?),
            ),
            Some(sleeper::Sleeper::SpinWait(_)) => Box::new(autd3_core::sleep::SpinWaitSleeper),
            Some(sleeper::Sleeper::Async(_)) => Box::new(autd3::r#async::controller::AsyncSleeper),
            None => Box::new(autd3::r#async::controller::AsyncSleeper),
        })
    }
}

impl From<&autd3_core::sleep::StdSleeper> for Sleeper {
    fn from(_: &autd3_core::sleep::StdSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::Std(crate::StdSleeper {})),
        }
    }
}

impl From<&autd3_core::sleep::SpinSleeper> for Sleeper {
    fn from(value: &autd3_core::sleep::SpinSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::Spin(crate::SpinSleeper {
                native_accuracy_ns: value.native_accuracy_ns(),
                spin_strategy: SpinStrategy::from(value.spin_strategy()) as _,
            })),
        }
    }
}

impl From<&autd3_core::sleep::SpinWaitSleeper> for Sleeper {
    fn from(_: &autd3_core::sleep::SpinWaitSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::SpinWait(crate::SpinWaitSleeper {})),
        }
    }
}

impl From<&autd3::r#async::controller::AsyncSleeper> for Sleeper {
    fn from(_: &autd3::r#async::controller::AsyncSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::Async(crate::AsyncSleeper {})),
        }
    }
}
