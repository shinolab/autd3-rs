use spin_sleep::SpinSleeper;
#[cfg(target_os = "windows")]
use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};

pub trait Sleeper {
    type Instant: super::instant::Instant;

    fn sleep_until(&self, deadline: Self::Instant) -> impl std::future::Future<Output = ()>;
}

pub struct StdSleeper {}

impl Sleeper for StdSleeper {
    type Instant = std::time::Instant;

    async fn sleep_until(&self, deadline: Self::Instant) {
        std::thread::sleep(deadline - std::time::Instant::now());
    }
}

impl Sleeper for SpinSleeper {
    type Instant = std::time::Instant;

    async fn sleep_until(&self, deadline: Self::Instant) {
        self.sleep(deadline - std::time::Instant::now());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsyncSleeper {
    #[cfg(target_os = "windows")]
    pub timer_resolution: Option<std::num::NonZeroU32>,
}

impl Sleeper for AsyncSleeper {
    type Instant = tokio::time::Instant;

    async fn sleep_until(&self, deadline: Self::Instant) {
        #[cfg(target_os = "windows")]
        self.timer_resolution
            .map(|timer_resolution| unsafe { timeBeginPeriod(timer_resolution.get()) });
        tokio::time::sleep_until(deadline).await;
        #[cfg(target_os = "windows")]
        self.timer_resolution
            .map(|timer_resolution| unsafe { timeEndPeriod(timer_resolution.get()) });
    }
}

#[cfg_attr(not(target_os = "windows"), allow(clippy::derivable_impls))]
impl Default for AsyncSleeper {
    fn default() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            timer_resolution: Some(std::num::NonZeroU32::MIN),
        }
    }
}
