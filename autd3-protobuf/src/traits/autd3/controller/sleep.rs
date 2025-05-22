use std::num::NonZeroU32;

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
            Some(sleeper::Sleeper::Std(std_sleeper)) => Box::new(autd3::controller::StdSleeper {
                timer_resolution: std_sleeper.timer_resolution.and_then(NonZeroU32::new),
            }),
            Some(sleeper::Sleeper::Spin(spin_sleeper)) => Box::new(
                autd3::controller::SpinSleeper::new(spin_sleeper.native_accuracy_ns)
                    .with_spin_strategy(autd3::controller::SpinStrategy::from_msg(
                        spin_sleeper.spin_strategy,
                    )?),
            ),
            #[cfg(target_os = "windows")]
            Some(sleeper::Sleeper::Waitable(_)) => Box::new(
                autd3::controller::WaitableSleeper::new()
                    .map_err(|_| AUTDProtoBufError::Unknown("WaitableTimer".to_string()))?,
            ),
            #[cfg(not(target_os = "windows"))]
            Some(sleeper::Sleeper::Waitable(_)) => {
                return Err(AUTDProtoBufError::NotSupportedData);
            }
            Some(sleeper::Sleeper::Async(async_sleeper)) => {
                Box::new(autd3::r#async::controller::AsyncSleeper {
                    timer_resolution: async_sleeper.timer_resolution.and_then(NonZeroU32::new),
                })
            }
            None => Box::new(autd3::r#async::controller::AsyncSleeper::default()),
        })
    }
}

impl From<&autd3::controller::StdSleeper> for Sleeper {
    fn from(value: &autd3::controller::StdSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::Std(crate::StdSleeper {
                timer_resolution: value.timer_resolution.map(|t| t.get()),
            })),
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

#[cfg(target_os = "windows")]
impl From<&autd3::controller::WaitableSleeper> for Sleeper {
    fn from(_: &autd3::controller::WaitableSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::Waitable(crate::WaitableSleeper {})),
        }
    }
}

impl From<&autd3::r#async::controller::AsyncSleeper> for Sleeper {
    fn from(value: &autd3::r#async::controller::AsyncSleeper) -> Self {
        Self {
            sleeper: Some(sleeper::Sleeper::Async(crate::AsyncSleeper {
                timer_resolution: value.timer_resolution.map(|t| t.get()),
            })),
        }
    }
}
