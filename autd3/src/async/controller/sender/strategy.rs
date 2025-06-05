use std::time::{Duration, Instant};

use super::sleep::AsyncSleep;
use crate::controller::{FixedDelay, FixedSchedule};

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    /// A trait for timer strategies.
    #[autd3_core::async_trait]
    pub trait AsyncTimerStrategy<S: AsyncSleep>: Send + Sync {
        /// Returns the initial instant.
        fn initial() -> Instant;
        /// Sleep until the specified time.
        /// The first call receives the return value of [`AsyncTimerStrategy::initialize`] as `old`, and subsequent calls receive the previous return value.
        async fn sleep(&self, old: Instant, interval: Duration) -> Instant;
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    /// A trait for timer strategies.
    pub trait AsyncTimerStrategy<S: AsyncSleep> {
        /// Returns the initial instant.
        fn initial() -> Instant;
        /// Sleep until the specified time.
        /// The first call receives the return value of [`AsyncTimerStrategy::initialize`] as `old`, and subsequent calls receive the previous return value.
        fn sleep(
            &self,
            old: Instant,
            interval: Duration,
        ) -> impl std::future::Future<Output = Instant> + Send;
    }
}

pub use internal::*;

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl<S: AsyncSleep> AsyncTimerStrategy<S> for FixedSchedule<S> {
    fn initial() -> Instant {
        Instant::now()
    }

    async fn sleep(&self, old: Instant, interval: Duration) -> Instant {
        let new = old + interval;
        self.0
            .sleep(new.saturating_duration_since(Instant::now()))
            .await;
        new
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl<S: AsyncSleep> AsyncTimerStrategy<S> for FixedDelay<S> {
    fn initial() -> Instant {
        Instant::now()
    }

    async fn sleep(&self, old: Instant, interval: Duration) -> Instant {
        self.0.sleep(interval).await;
        old
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use derive_more::Debug;

    use super::*;

    #[derive(Debug)]
    struct DebugSleep {
        sleep: Arc<RwLock<Vec<Duration>>>,
    }

    #[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
    impl AsyncSleep for DebugSleep {
        async fn sleep(&self, duration: Duration) {
            self.sleep.write().unwrap().push(duration);
        }
    }

    #[tokio::test]
    async fn fixed_schedule_test() {
        let sleep = Arc::new(RwLock::new(Vec::new()));

        let strategy = FixedSchedule(DebugSleep { sleep });

        let start = FixedSchedule::<DebugSleep>::initial();
        let interval = Duration::from_millis(1);

        let next = strategy.sleep(start, interval).await;
        assert_eq!(next, start + interval);

        let next = strategy.sleep(next, interval).await;
        assert_eq!(next, start + interval * 2);
    }

    #[tokio::test]
    async fn fixed_delay_test() {
        let sleep = Arc::new(RwLock::new(Vec::new()));

        let strategy = FixedDelay(DebugSleep {
            sleep: sleep.clone(),
        });

        let start = FixedDelay::<DebugSleep>::initial();
        let interval = Duration::from_millis(1);

        let next = strategy.sleep(start, interval).await;
        assert_eq!(next, start);

        let next = strategy.sleep(start, interval).await;
        assert_eq!(next, start);

        assert_eq!(*sleep.read().unwrap(), vec![interval, interval]);
    }
}
