use std::time::Duration;

pub use spin_sleep::{SpinSleeper, SpinStrategy};

/// A trait for sleep operations.
pub trait Sleep: std::fmt::Debug {
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

/// A sleeper that uses [`std::thread::sleep`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StdSleeper;

impl Sleep for StdSleeper {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

impl Sleep for SpinSleeper {
    fn sleep(&self, duration: Duration) {
        SpinSleeper::sleep(*self, duration);
    }
}

/// A sleeper that uses a spin loop to wait until the deadline is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpinWaitSleeper;

impl Sleep for SpinWaitSleeper {
    fn sleep(&self, duration: Duration) {
        use std::time::Instant;

        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            std::hint::spin_loop();
        }
    }
}
