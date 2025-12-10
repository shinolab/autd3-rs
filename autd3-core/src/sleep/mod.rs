use core::time::Duration;

#[cfg(target_os = "windows")]
unsafe extern "system" {
    fn timeBeginPeriod(u: u32) -> u32;
    fn timeEndPeriod(u: u32) -> u32;
}

/// A trait for sleep operations.
pub trait Sleeper {
    /// Sleep for the specified duration.
    fn sleep(&self, duration: Duration);
}

/// A trait for sleep operations.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub trait AsyncSleeper {
    /// Sleep for the specified duration.
    fn sleep(&self, duration: Duration) -> impl std::future::Future<Output = ()>;
}

impl Sleeper for Box<dyn Sleeper> {
    fn sleep(&self, duration: Duration) {
        self.as_ref().sleep(duration);
    }
}

/// A sleeper that uses [`std::thread::sleep`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StdSleeper;

impl Sleeper for StdSleeper {
    fn sleep(&self, duration: Duration) {
        if duration.is_zero() {
            return;
        }

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
    fn sleep(&self, duration: Duration) {
        use std::time::Instant;

        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            core::hint::spin_loop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_sleeper() {
        let sleeper = StdSleeper;
        {
            let start = std::time::Instant::now();
            sleeper.sleep(Duration::from_millis(10));
            assert!(Duration::from_millis(10) <= start.elapsed());
        }
        {
            let start = std::time::Instant::now();
            sleeper.sleep(Duration::from_millis(0));
            assert!(Duration::from_millis(0) <= start.elapsed());
        }
    }

    #[test]
    fn spin_wait_sleeper() {
        let sleeper = SpinWaitSleeper;
        {
            let start = std::time::Instant::now();
            sleeper.sleep(Duration::from_millis(10));
            assert!(Duration::from_millis(10) <= start.elapsed());
        }
        {
            let start = std::time::Instant::now();
            sleeper.sleep(Duration::from_millis(0));
            assert!(Duration::from_millis(0) <= start.elapsed());
        }
    }

    #[test]
    fn box_sleeper() {
        let sleeper: Box<dyn Sleeper> = Box::new(StdSleeper);
        {
            let start = std::time::Instant::now();
            sleeper.sleep(Duration::from_millis(10));
            assert!(Duration::from_millis(10) <= start.elapsed());
        }
        {
            let start = std::time::Instant::now();
            sleeper.sleep(Duration::from_millis(0));
            assert!(Duration::from_millis(0) <= start.elapsed());
        }
    }
}
