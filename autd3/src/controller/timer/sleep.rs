use autd3_driver::utils::timer::TimerResolutionGurad;
use spin_sleep::SpinSleeper;

pub(crate) trait Sleeper {
    type Instant: super::instant::Instant;

    fn sleep_until(&self, deadline: Self::Instant) -> impl std::future::Future<Output = ()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdSleeper {
    pub timer_resolution: Option<std::num::NonZeroU32>,
}

impl Default for StdSleeper {
    fn default() -> Self {
        Self {
            timer_resolution: Some(std::num::NonZeroU32::MIN),
        }
    }
}

impl Sleeper for StdSleeper {
    type Instant = std::time::Instant;

    async fn sleep_until(&self, deadline: Self::Instant) {
        let _timer_guard = TimerResolutionGurad::new(self.timer_resolution);
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
    pub timer_resolution: Option<std::num::NonZeroU32>,
}

impl Default for AsyncSleeper {
    fn default() -> Self {
        Self {
            timer_resolution: Some(std::num::NonZeroU32::MIN),
        }
    }
}

impl Sleeper for AsyncSleeper {
    type Instant = tokio::time::Instant;

    async fn sleep_until(&self, deadline: Self::Instant) {
        let _timer_guard = TimerResolutionGurad::new(self.timer_resolution);
        tokio::time::sleep_until(deadline).await;
    }
}

#[cfg(target_os = "windows")]
pub use win::WaitableSleeper;

#[cfg(target_os = "windows")]
mod win {
    use super::Sleeper;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WaitableSleeper {
        handle: windows::Win32::Foundation::HANDLE,
    }

    unsafe impl Send for WaitableSleeper {}
    unsafe impl Sync for WaitableSleeper {}

    impl WaitableSleeper {
        pub fn new() -> windows::core::Result<Self> {
            Ok(Self {
                handle: unsafe {
                    windows::Win32::System::Threading::CreateWaitableTimerExW(
                        None,
                        None,
                        windows::Win32::System::Threading::CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                        windows::Win32::System::Threading::TIMER_ALL_ACCESS.0,
                    )?
                },
            })
        }
    }

    impl Sleeper for WaitableSleeper {
        type Instant = std::time::Instant;

        async fn sleep_until(&self, deadline: Self::Instant) {
            unsafe {
                let time = deadline - std::time::Instant::now();
                if time.is_zero() {
                    return;
                }
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
    }
}
