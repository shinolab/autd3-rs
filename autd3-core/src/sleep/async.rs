use std::time::Duration;

use super::{SpinSleeper, SpinWaitSleeper, StdSleeper};

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    #[doc(hidden)]
    #[async_trait::async_trait]
    pub trait AsyncSleep: std::fmt::Debug + Send + Sync {
        async fn sleep(&self, duration: Duration);
    }

    #[async_trait::async_trait]
    impl AsyncSleep for Box<dyn AsyncSleep> {
        async fn sleep(&self, duration: Duration) {
            self.as_ref().sleep(duration).await;
        }
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    #[doc(hidden)]
    pub trait AsyncSleep: std::fmt::Debug + Send + Sync {
        fn sleep(&self, duration: Duration) -> impl std::future::Future<Output = ()> + Send;
    }
}

pub use internal::*;

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncSleep for StdSleeper {
    async fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncSleep for SpinSleeper {
    async fn sleep(&self, duration: Duration) {
        SpinSleeper::sleep(*self, duration);
    }
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncSleep for SpinWaitSleeper {
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
impl AsyncSleep for AsyncSleeper {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
