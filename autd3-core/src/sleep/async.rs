use std::time::Duration;

use super::SpinWaitSleeper;

#[doc(hidden)]
pub trait Sleep: std::fmt::Debug + Send + Sync {
    fn sleep(&self, duration: Duration) -> impl std::future::Future<Output = ()> + Send;
}

impl Sleep for SpinWaitSleeper {
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
