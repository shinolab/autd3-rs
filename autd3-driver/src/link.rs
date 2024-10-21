use crate::{
    error::AUTDInternalError,
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
        async fn close(&mut self) -> Result<(), AUTDInternalError>;

        async fn update(&mut self, _geometry: &Geometry) -> Result<(), AUTDInternalError> {
            Ok(())
        }

        async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDInternalError>;

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError>;

        #[must_use]
        fn is_open(&self) -> bool;
    }

    #[async_trait::async_trait]
    pub trait LinkBuilder: Send + Sync {
        type L: Link;

        async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError>;
    }

    #[async_trait::async_trait]
    impl Link for Box<dyn Link> {
        async fn close(&mut self) -> Result<(), AUTDInternalError> {
            self.as_mut().close().await
        }

        async fn update(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
            self.as_mut().update(geometry).await
        }

        async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDInternalError> {
            self.as_mut().send(tx).await
        }

        async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
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

    pub trait Link: Send {
        fn close(&mut self) -> impl std::future::Future<Output = Result<(), AUTDInternalError>>;

        fn update(
            &mut self,
            _geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<(), AUTDInternalError>> {
            async { Ok(()) }
        }

        fn send(
            &mut self,
            tx: &[TxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>>;

        fn receive(
            &mut self,
            rx: &mut [RxMessage],
        ) -> impl std::future::Future<Output = Result<bool, AUTDInternalError>>;

        #[must_use]
        fn is_open(&self) -> bool;
    }

    pub trait LinkBuilder {
        type L: Link;

        fn open(
            self,
            geometry: &Geometry,
        ) -> impl std::future::Future<Output = Result<Self::L, AUTDInternalError>>;
    }
}
