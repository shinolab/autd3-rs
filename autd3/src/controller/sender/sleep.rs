use std::time::Instant;

pub use spin_sleep::SpinSleeper;

/// A trait for sleep operations.
pub trait Sleep: std::fmt::Debug {
    /// Sleep until the specified deadline.
    fn sleep_until(&self, deadline: Instant);
}

// GRCOV_EXCL_START
impl Sleep for Box<dyn Sleep> {
    fn sleep_until(&self, deadline: Instant) {
        self.as_ref().sleep_until(deadline);
    }
}
// GRCOV_EXCL_STOP

/// A sleeper that uses [`std::thread::sleep`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StdSleeper;

impl Sleep for StdSleeper {
    fn sleep_until(&self, deadline: Instant) {
        std::thread::sleep(deadline - Instant::now());
    }
}

impl Sleep for SpinSleeper {
    fn sleep_until(&self, deadline: Instant) {
        self.sleep(deadline - Instant::now());
    }
}
