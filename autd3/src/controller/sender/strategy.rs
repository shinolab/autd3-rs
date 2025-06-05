use std::time::{Duration, Instant};

use autd3_core::sleep::Sleep;

/// A trait for timer strategies.
pub trait TimerStrategy<S: Sleep> {
    /// Instant type used by the timer strategy.
    type Instant;

    /// Returns the initial instant.
    fn initial() -> Self::Instant;
    /// Sleep until the specified time.
    /// The first call receives the return value of [`TimerStrategy::initial`] as `old`, and subsequent calls receive the previous return value.
    fn sleep(&self, old: Self::Instant, interval: Duration) -> Self::Instant;
}

/// [`FixedSchedule`] prioritize average behavior for the transmission timing. That is, not the interval from the previous transmission, but ensuring that T/interval transmissions are performed in a sufficiently long time T.
// For example, if the interval is 1ms and it takes 1.5ms to transmit due to some reason, the next transmission will be performed not 1ms later but 0.5ms later.
pub struct FixedSchedule<S>(pub S);

impl<S: Sleep> TimerStrategy<S> for FixedSchedule<S> {
    type Instant = Instant;

    fn initial() -> Self::Instant {
        Instant::now()
    }

    fn sleep(&self, old: Self::Instant, interval: Duration) -> Self::Instant {
        let new = old + interval;
        self.0.sleep(new.saturating_duration_since(Instant::now()));
        new
    }
}

impl Default for FixedSchedule<autd3_core::sleep::SpinSleeper> {
    fn default() -> Self {
        Self(autd3_core::sleep::SpinSleeper::default())
    }
}

/// [`FixedDelay`] prioritize the delay from the previous transmission. That is, it sleeps for the specified interval regardless of the time taken for the previous transmission.
pub struct FixedDelay<S>(pub S);

impl<S: Sleep> TimerStrategy<S> for FixedDelay<S> {
    type Instant = Instant;

    fn initial() -> Self::Instant {
        Instant::now()
    }

    fn sleep(&self, old: Self::Instant, interval: Duration) -> Self::Instant {
        self.0.sleep(interval);
        old
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use derive_more::Debug;

    use super::*;

    #[derive(Debug)]
    struct DebugSleep {
        sleep: Rc<RefCell<Vec<Duration>>>,
    }

    impl Sleep for DebugSleep {
        fn sleep(&self, duration: Duration) {
            self.sleep.borrow_mut().push(duration);
        }
    }

    #[test]
    fn fixed_schedule_test() {
        let sleep = Rc::new(RefCell::new(Vec::new()));

        let strategy = FixedSchedule(DebugSleep { sleep });

        let start = FixedSchedule::<DebugSleep>::initial();
        let interval = Duration::from_millis(1);

        let next = strategy.sleep(start, interval);
        assert_eq!(next, start + interval);

        let next = strategy.sleep(next, interval);
        assert_eq!(next, start + interval * 2);
    }

    #[test]
    fn fixed_delay_test() {
        let sleep = Rc::new(RefCell::new(Vec::new()));

        let strategy = FixedDelay(DebugSleep {
            sleep: sleep.clone(),
        });

        let start = FixedDelay::<DebugSleep>::initial();
        let interval = Duration::from_millis(1);

        let next = strategy.sleep(start, interval);
        assert_eq!(next, start);

        let next = strategy.sleep(start, interval);
        assert_eq!(next, start);

        assert_eq!(*sleep.borrow(), vec![interval, interval]);
    }
}
