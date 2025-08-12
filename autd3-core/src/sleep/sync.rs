use core::time::Duration;

use alloc::boxed::Box;

#[cfg(feature = "std")]
pub use spin_sleep::{SpinSleeper, SpinStrategy};

/// A trait for sleep operations.
pub trait Sleep: core::fmt::Debug {
    /// Sleep until the specified deadline.
    fn sleep(&self, duration: Duration);
}

// GRCOV_EXCL_START
impl Sleep for Box<dyn Sleep> {
    fn sleep(&self, duration: Duration) {
        self.as_ref().sleep(duration);
    }
}
// GRCOV_EXCL_STOP

#[cfg(feature = "std")]
/// A sleeper that uses [`std::thread::sleep`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StdSleeper;

#[cfg(feature = "std")]
impl Sleep for StdSleeper {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

#[cfg(feature = "std")]
impl Sleep for SpinSleeper {
    fn sleep(&self, duration: Duration) {
        SpinSleeper::sleep(*self, duration);
    }
}

#[cfg(feature = "std")]
/// A sleeper that uses a spin loop to wait until the deadline is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpinWaitSleeper;

#[cfg(feature = "std")]
impl Sleep for SpinWaitSleeper {
    fn sleep(&self, duration: Duration) {
        use std::time::Instant;

        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            core::hint::spin_loop();
        }
    }
}
