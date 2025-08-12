use core::time::Duration;

#[doc(hidden)]
pub trait Sleep: core::fmt::Debug + Send + Sync {
    fn sleep(&self, duration: Duration) -> impl core::future::Future<Output = ()> + Send;
}

impl Sleep for super::SpinWaitSleeper {
    async fn sleep(&self, duration: Duration) {
        use std::time::Instant;

        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            tokio::task::yield_now().await;
        }
    }
}

/// A sleeper that uses [`tokio::time::sleep_until`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AsyncSleeper;

impl Sleep for AsyncSleeper {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
