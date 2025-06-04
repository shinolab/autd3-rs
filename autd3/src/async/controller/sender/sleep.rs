use std::time::Instant;

pub use spin_sleep::SpinSleeper;

use crate::controller::StdSleeper;

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    #[doc(hidden)]
    #[autd3_core::async_trait]
    pub trait AsyncSleep: std::fmt::Debug {
        async fn sleep_until(&self, deadline: Instant);
    }

    #[autd3_core::async_trait]
    impl AsyncSleep for Box<dyn AsyncSleep + Send + Sync> {
        async fn sleep_until(&self, deadline: Instant) {
            self.as_ref().sleep_until(deadline).await;
        }
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    #[doc(hidden)]
    pub trait AsyncSleep: std::fmt::Debug {
        fn sleep_until(&self, deadline: Instant) -> impl std::future::Future<Output = ()> + Send;
    }
}

pub use internal::*;

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncSleep for StdSleeper {
    async fn sleep_until(&self, deadline: Instant) {
        std::thread::sleep(deadline - Instant::now());
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncSleep for SpinSleeper {
    async fn sleep_until(&self, deadline: Instant) {
        self.sleep(deadline - Instant::now());
    }
}

/// A sleeper that uses [`tokio::time::sleep_until`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AsyncSleeper;

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncSleep for AsyncSleeper {
    async fn sleep_until(&self, deadline: Instant) {
        tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
    }
}
