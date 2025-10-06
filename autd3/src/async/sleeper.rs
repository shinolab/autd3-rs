use std::time::Duration;

use autd3_core::sleep::r#async::Sleep;

/// A sleeper that uses [`tokio::time::sleep_until`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AsyncSleeper;

impl Sleep for AsyncSleeper {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
