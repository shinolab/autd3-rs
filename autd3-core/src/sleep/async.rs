use core::time::Duration;

/// A trait for asynchronous sleep operations.
pub trait Sleep: Send + Sync {
    fn sleep(&self, duration: Duration) -> impl core::future::Future<Output = ()> + Send;
}
