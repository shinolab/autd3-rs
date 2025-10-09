use core::time::Duration;

#[cfg(target_os = "windows")]
unsafe extern "system" {
    fn timeBeginPeriod(u: u32) -> u32;
    fn timeEndPeriod(u: u32) -> u32;
}

/// A trait for sleep operations.
pub trait Sleeper {
    /// Sleep until the specified deadline.
    fn sleep(&self, duration: Duration);
}

// GRCOV_EXCL_START
impl Sleeper for Box<dyn Sleeper> {
    fn sleep(&self, duration: Duration) {
        self.as_ref().sleep(duration);
    }
}
// GRCOV_EXCL_STOP

/// A sleeper that uses [`std::thread::sleep`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StdSleeper;

impl Sleeper for StdSleeper {
    fn sleep(&self, duration: Duration) {
        #[cfg(target_os = "windows")]
        unsafe {
            timeBeginPeriod(1);
        }
        std::thread::sleep(duration);
        #[cfg(target_os = "windows")]
        unsafe {
            timeEndPeriod(1);
        }
    }
}

/// A sleeper that uses a spin loop to wait until the deadline is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpinWaitSleeper;

impl Sleeper for SpinWaitSleeper {
    // GRCOV_EXCL_START
    fn sleep(&self, duration: Duration) {
        use std::time::Instant;

        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            core::hint::spin_loop();
        }
    }
    // GRCOV_EXCL_STOP
}
