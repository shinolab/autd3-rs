use std::time::Duration;

use crate::{
    error::AUTDDriverError,
    firmware::cpu::{RxMessage, TxMessage},
    geometry::Geometry,
};

pub use internal::Link;
pub use internal::LinkBuilder;

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    #[async_trait::async_trait]
    pub trait Link: Send {
        async fn close(&mut self) -> Result<(), AUTDDriverError>;

        async fn update(&mut self, _geometry: &Geometry) -> Result<(), AUTDDriverError> {
            Ok(())
        }

        async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError>;

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError>;

        #[must_use]
        fn is_open(&self) -> bool;

        fn trace(&mut self, _: Option<Duration>, _: Option<usize>) {}
    }

    #[async_trait::async_trait]
    pub trait LinkBuilder: Send + Sync {
        type L: Link;

        async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDDriverError>;
    }

    #[async_trait::async_trait]
    impl Link for Box<dyn Link> {
        async fn close(&mut self) -> Result<(), AUTDDriverError> {
            self.as_mut().close().await
        }

        async fn update(&mut self, geometry: &Geometry) -> Result<(), AUTDDriverError> {
            self.as_mut().update(geometry).await
        }

        async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError> {
            self.as_mut().send(tx).await
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError> {
            self.as_mut().receive(rx).await
        }

        fn is_open(&self) -> bool {
            self.as_ref().is_open()
        }

        fn trace(&mut self, timeout: Option<Duration>, parallel_threshold: Option<usize>) {
            self.as_mut().trace(timeout, parallel_threshold)
        }
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    pub trait Link: Send {
        fn close(&mut self) -> impl std::future::Future<Output = Result<(), AUTDDriverError>>;

        fn update(
            &mut self,
            _geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<(), AUTDDriverError>> {
            async { Ok(()) }
        }

        fn send(
            &mut self,
            tx: &[TxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDDriverError>>;

        fn receive(
            &mut self,
            rx: &mut [RxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDDriverError>>;

        #[must_use]
        fn is_open(&self) -> bool;

        fn trace(&mut self, _: Option<Duration>, _: Option<usize>) {}
    }

    pub trait LinkBuilder {
        type L: Link;

        fn open(
            self,
            geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<Self::L, AUTDDriverError>>;
    }
}
