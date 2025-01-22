use std::time::Instant;

use autd3_core::utils::timer::TimerResolutionGurad;
pub use spin_sleep::SpinSleeper;

pub trait Sleep {
    fn sleep_until(&self, deadline: Instant);
}

/// A sleeper that uses [`std::thread::sleep`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdSleeper {
    /// An optional timer resolution in milliseconds for Windows. The default is `Some(1)`.
    pub timer_resolution: Option<std::num::NonZeroU32>,
}

impl Default for StdSleeper {
    fn default() -> Self {
        Self {
            timer_resolution: Some(std::num::NonZeroU32::MIN),
        }
    }
}

impl Sleep for StdSleeper {
    fn sleep_until(&self, deadline: Instant) {
        let _timer_guard = TimerResolutionGurad::new(self.timer_resolution);
        std::thread::sleep(deadline - Instant::now());
    }
}

impl Sleep for SpinSleeper {
    fn sleep_until(&self, deadline: Instant) {
        self.sleep(deadline - Instant::now());
    }
}

#[cfg(target_os = "windows")]
pub use win::WaitableSleeper;

#[cfg(target_os = "windows")]
mod win {
    use super::*;

    /// A sleeper that uses [waitable timer](https://learn.microsoft.com/en-us/windows/win32/sync/waitable-timer-objects) available only on Windows.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WaitableSleeper {
        pub(crate) handle: windows::Win32::Foundation::HANDLE,
    }

    unsafe impl Send for WaitableSleeper {}
    unsafe impl Sync for WaitableSleeper {}

    impl WaitableSleeper {
        /// Creates a new [`WaitableSleeper`].
        ///
        /// # Errors
        ///
        /// See [`CreateWaitableTimerExW`] for more details.
        ///
        /// [`CreateWaitableTimerExW`]: https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createwaitabletimerexw
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

    impl Sleep for WaitableSleeper {
        // GRCOV_EXCL_START
        fn sleep_until(&self, deadline: Instant) {
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
