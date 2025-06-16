use std::time::Duration;

use super::SpinWaitSleeper;

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    #[doc(hidden)]
    #[async_trait::async_trait]
    pub trait Sleep: std::fmt::Debug + Send + Sync {
        async fn sleep(&self, duration: Duration);
    }

    #[async_trait::async_trait]
    impl Sleep for Box<dyn Sleep> {
        async fn sleep(&self, duration: Duration) {
            self.as_ref().sleep(duration).await;
        }
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    #[doc(hidden)]
    pub trait Sleep: std::fmt::Debug + Send + Sync {
        fn sleep(&self, duration: Duration) -> impl std::future::Future<Output = ()> + Send;
    }
}

pub use internal::*;

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
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

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Sleep for AsyncSleeper {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
