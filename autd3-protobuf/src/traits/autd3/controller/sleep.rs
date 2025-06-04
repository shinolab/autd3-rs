use crate::{AUTDProtoBufError, FromMessage, Sleeper, SpinStrategy, sleeper};

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

impl FromMessage<Sleeper> for Box<dyn autd3::r#async::controller::AsyncSleep + Send + Sync> {
    fn from_msg(value: Sleeper) -> Result<Self, AUTDProtoBufError> {
        Ok(match value.sleeper {
            Some(sleeper::Sleeper::Std(_)) => Box::new(autd3::controller::StdSleeper),
            Some(sleeper::Sleeper::Spin(spin_sleeper)) => Box::new(
                autd3::controller::SpinSleeper::new(spin_sleeper.native_accuracy_ns)
                    .with_spin_strategy(autd3::controller::SpinStrategy::from_msg(
                        spin_sleeper.spin_strategy,
                    )?),
            ),
            Some(sleeper::Sleeper::SpinWait(_)) => Box::new(autd3::controller::SpinWaitSleeper),
            Some(sleeper::Sleeper::Async(_)) => Box::new(autd3::r#async::controller::AsyncSleeper),
            None => Box::new(autd3::r#async::controller::AsyncSleeper),
        })
    }
}

impl From<&autd3::controller::StdSleeper> for Sleeper {
    fn from(_: &autd3::controller::StdSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::Std(crate::StdSleeper {})),
        }
    }
}

impl From<&autd3::controller::SpinSleeper> for Sleeper {
    fn from(value: &autd3::controller::SpinSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::Spin(crate::SpinSleeper {
                native_accuracy_ns: value.native_accuracy_ns(),
                spin_strategy: SpinStrategy::from(value.spin_strategy()) as _,
            })),
        }
    }
}

impl From<&autd3::controller::SpinWaitSleeper> for Sleeper {
    fn from(_: &autd3::controller::SpinWaitSleeper) -> Self {
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
