use std::time::Instant;

use autd3_core::utils::timer::TimerResolutionGurad;
pub use spin_sleep::SpinSleeper;

use crate::controller::StdSleeper;

#[doc(hidden)]
#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
pub trait AsyncSleep: std::fmt::Debug {
    async fn sleep_until(&self, deadline: Instant);
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncSleep for Box<dyn AsyncSleep + Send + Sync> {
    async fn sleep_until(&self, deadline: Instant) {
        self.as_ref().sleep_until(deadline).await;
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncSleep for StdSleeper {
    async fn sleep_until(&self, deadline: Instant) {
        let _timer_guard = TimerResolutionGurad::new(self.timer_resolution);
        std::thread::sleep(deadline - Instant::now());
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncSleep for SpinSleeper {
    async fn sleep_until(&self, deadline: Instant) {
        self.sleep(deadline - Instant::now());
    }
}

/// A sleeper that uses [`tokio::time::sleep_until`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsyncSleeper {
    /// An optional timer resolution in milliseconds for Windows. The default is `Some(1)`.
    pub timer_resolution: Option<std::num::NonZeroU32>,
}

impl Default for AsyncSleeper {
    fn default() -> Self {
        Self {
            timer_resolution: Some(std::num::NonZeroU32::MIN),
        }
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncSleep for AsyncSleeper {
    async fn sleep_until(&self, deadline: Instant) {
        let _timer_guard = TimerResolutionGurad::new(self.timer_resolution);
        tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
    }
}

#[cfg(target_os = "windows")]
mod win {
    use crate::controller::WaitableSleeper;

    use super::*;

    #[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
    impl AsyncSleep for WaitableSleeper {
        // GRCOV_EXCL_START
        async fn sleep_until(&self, deadline: Instant) {
            unsafe {
                let time = deadline - Instant::now();
                if time.is_zero() {
                    return;
                }
                // The unit of SetWaitableTimer is 100ns and negative value means relative time.
                // See [SetWaitableTimer](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer) for more details.
                let duetime = (time.as_nanos() / 100) as i64;
                let duetime = -duetime;
                let set_and_wait = || {
                    if let Err(e) = windows::Win32::System::Threading::SetWaitableTimer(
                        self.handle,
                        &duetime,
                        0,
                        None,
                        None,
                        false,
                    ) {
                        tracing::warn!(
                            "SetWaitableTimer failed: {:?}, fallback to std::thread::sleep...",
                            e
                        );
                        return false;
                    }
                    if windows::Win32::System::Threading::WaitForSingleObject(
                        self.handle,
                        windows::Win32::System::Threading::INFINITE,
                    ) == windows::Win32::Foundation::WAIT_FAILED
                    {
                        tracing::warn!(
                            "WaitForSingleObject failed: {:?}, fallback to std::thread::sleep...",
                            windows::Win32::Foundation::GetLastError()
                        );
                        return false;
                    }
                    true
                };
                if !set_and_wait() {
                    let _timer_guard = super::TimerResolutionGurad::new(Some(
                        std::num::NonZeroU32::new(1).unwrap(),
                    ));
                    std::thread::sleep(time);
                }
            }
        }
        // GRCOV_EXCL_STOP
    }
}
