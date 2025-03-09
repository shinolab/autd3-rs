use std::num::NonZeroU32;

use crate::{AUTDProtoBufError, FromMessage, SpinStrategy, sender_option::Sleeper};

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
        match value {
            Sleeper::Std(std_sleeper) => Ok(Box::new(autd3::controller::StdSleeper {
                timer_resolution: std_sleeper.timer_resolution.and_then(NonZeroU32::new),
            })),
            Sleeper::Spin(spin_sleeper) => Ok(Box::new(
                autd3::controller::SpinSleeper::new(spin_sleeper.native_accuracy_ns)
                    .with_spin_strategy(autd3::controller::SpinStrategy::from_msg(
                        spin_sleeper.spin_strategy,
                    )?),
            )),
            #[cfg(target_os = "windows")]
            Sleeper::Waitable(_) => Ok(Box::new(
                autd3::controller::WaitableSleeper::new().map_err(|_| {
                    AUTDProtoBufError::Status(tonic::Status::unknown("WaitableSleeper"))
                })?,
            )),
            #[cfg(not(target_os = "windows"))]
            Sleeper::Waitable(_) => Err(AUTDProtoBufError::Status(tonic::Status::unimplemented(
                "WaitableSleeper is not supported",
            ))),
            Sleeper::Async(async_sleeper) => {
                Ok(Box::new(autd3::r#async::controller::AsyncSleeper {
                    timer_resolution: async_sleeper.timer_resolution.and_then(NonZeroU32::new),
                }))
            }
        }
    }
}
