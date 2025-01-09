use std::time::Duration;

use crate::{
    error::AUTDDriverError,
    firmware::cpu::{RxMessage, TxMessage},
    geometry::Geometry,
};

pub use internal::{AsyncLink, AsyncLinkBuilder};

#[cfg(feature = "async-trait")]
mod internal {
    use super::*;

    /// A trait that provides the interface with the device.
    #[async_trait::async_trait]
    pub trait AsyncLink: Send {
        /// Closes the link.
        async fn close(&mut self) -> Result<(), AUTDDriverError>;

        #[doc(hidden)]
        async fn update(&mut self, _geometry: &Geometry) -> Result<(), AUTDDriverError> {
            Ok(())
        }

        /// Sends a message to the device.
        async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError>;

        /// Receives a message from the device.
        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError>;

        /// Checks if the link is open.
        #[must_use]
        fn is_open(&self) -> bool;

        #[doc(hidden)]
        fn trace(&mut self, _: Option<Duration>, _: Option<usize>) {}
    }

    /// A trait to build a link.
    #[async_trait::async_trait]
    pub trait AsyncLinkBuilder: Send + Sync {
        /// The link type.
        type L: AsyncLink;

        /// Opens a link.
        async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDDriverError>;
    }

    #[async_trait::async_trait]
    impl AsyncLink for Box<dyn AsyncLink> {
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

    /// A trait that provides the interface with the device.
    pub trait AsyncLink: Send {
        /// Closes the link.
        fn close(&mut self) -> impl std::future::Future<Output = Result<(), AUTDDriverError>>;

        #[doc(hidden)]
        fn update(
            &mut self,
            _geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<(), AUTDDriverError>> {
            async { Ok(()) }
        }

        /// Sends a message to the device.
        fn send(
            &mut self,
            tx: &[TxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDDriverError>>;

        /// Receives a message from the device.
        fn receive(
            &mut self,
            rx: &mut [RxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDDriverError>>;

        /// Checks if the link is open.
        #[must_use]
        fn is_open(&self) -> bool;

        #[doc(hidden)]
        fn trace(&mut self, _: Option<Duration>, _: Option<usize>) {}
    }

    /// A trait to build a link.
    pub trait AsyncLinkBuilder {
        /// The link type.
        type L: AsyncLink;

        /// Opens a link.
        fn open(
            self,
            geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<Self::L, AUTDDriverError>>;
    }
}
