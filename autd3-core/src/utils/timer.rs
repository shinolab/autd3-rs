use std::num::NonZeroU32;

/// A utility to set the timer resolution on Windows.
#[doc(hidden)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct TimerResolutionGurad {
    timer_resolution: Option<NonZeroU32>,
}

impl TimerResolutionGurad {
    #[must_use]
    pub fn new(timer_resolution: Option<NonZeroU32>) -> Self {
        #[cfg(target_os = "windows")]
        timer_resolution.map(|timer_resolution| unsafe {
            windows::Win32::Media::timeBeginPeriod(timer_resolution.get())
        });
        Self { timer_resolution }
    }
}

impl Drop for TimerResolutionGurad {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        self.timer_resolution.map(|timer_resolution| unsafe {
            windows::Win32::Media::timeEndPeriod(timer_resolution.get())
        });
    }
}
