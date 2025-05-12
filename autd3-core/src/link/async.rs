use crate::{
    geometry::Geometry,
    link::{LinkError, RxMessage, TxMessage},
};

pub use internal::AsyncLink;

#[cfg(feature = "async-trait")]
mod internal {

    use super::*;

    /// A trait that provides the interface with the device.
    #[async_trait::async_trait]
    pub trait AsyncLink: Send {
        /// Opens the link.
        async fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError>;

        /// Closes the link.
        async fn close(&mut self) -> Result<(), LinkError>;

        #[doc(hidden)]
        async fn update(&mut self, _: &Geometry) -> Result<(), LinkError> {
            Ok(())
        }

        /// Allocate a sending buffer for the link.
        async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError>;

        /// Sends a message to the device.
        async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError>;

        /// Receives a message from the device.
        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError>;

        /// Checks if the link is open.
        #[must_use]
        fn is_open(&self) -> bool;
    }

    #[async_trait::async_trait]
    impl AsyncLink for Box<dyn AsyncLink> {
        async fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
            self.as_mut().open(geometry).await
        }

        async fn close(&mut self) -> Result<(), LinkError> {
            self.as_mut().close().await
        }

        async fn update(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
            self.as_mut().update(geometry).await
        }

        async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
            self.as_mut().alloc_tx_buffer().await
        }

        async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
            self.as_mut().send(tx).await
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
            self.as_mut().receive(rx).await
        }

        fn is_open(&self) -> bool {
            self.as_ref().is_open()
        }
    }
}

#[cfg(not(feature = "async-trait"))]
mod internal {
    use super::*;

    /// A trait that provides the interface with the device.
    pub trait AsyncLink: Send {
        /// Opens the link.
        fn open(
            &mut self,
            geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<(), LinkError>>;

        /// Closes the link.
        fn close(&mut self) -> impl std::future::Future<Output = Result<(), LinkError>>;

        #[doc(hidden)]
        fn update(
            &mut self,
            _: &Geometry,
        ) -> impl std::future::Future<Output = Result<(), LinkError>> {
            async { Ok(()) }
        }

        /// Allocate a sending buffer for the link.
        fn alloc_tx_buffer(
            &mut self,
        ) -> impl std::future::Future<Output = Result<Vec<TxMessage>, LinkError>>;

        /// Sends a message to the device.
        fn send(
            &mut self,
            tx: Vec<TxMessage>,
        ) -> impl std::future::Future<Output = Result<(), LinkError>>;

        /// Receives a message from the device.
        fn receive(
            &mut self,
            rx: &mut [RxMessage],
        ) -> impl std::future::Future<Output = Result<(), LinkError>>;

        /// Checks if the link is open.
        #[must_use]
        fn is_open(&self) -> bool;
    }
}
