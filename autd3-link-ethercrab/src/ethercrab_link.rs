use autd3_core::{
    geometry::Geometry,
    link::{AsyncLink, Link, LinkError, RxMessage, TxMessage},
};

use crate::{
    Status,
    inner::{EtherCrabHandler, EtherCrabOptionFull},
};

/// A [`Link`] using [EtherCrab](https://github.com/ethercrab-rs/ethercrab).
pub struct EtherCrab<F: Fn(usize, Status) + Send + Sync + 'static> {
    option: Option<(F, EtherCrabOptionFull)>,
    inner: Option<EtherCrabHandler>,
    #[cfg(feature = "tokio")]
    runtime: Option<tokio::runtime::Runtime>,
}

impl<F: Fn(usize, Status) + Send + Sync + 'static> EtherCrab<F> {
    /// Creates a new [`EtherCrab`]
    pub fn new(err_handler: F, option: impl Into<EtherCrabOptionFull>) -> Self {
        Self {
            option: Some((err_handler, option.into())),
            inner: None,
            #[cfg(feature = "tokio")]
            runtime: None,
        }
    }

    /// Creates a new [`EtherCrab`] with the given [`tokio::runtime::Runtime`].
    #[cfg(feature = "tokio")]
    pub fn with_runtime(
        err_handler: F,
        option: impl Into<EtherCrabOptionFull>,
        runtime: tokio::runtime::Runtime,
    ) -> Self {
        Self {
            option: Some((err_handler, option.into())),
            inner: None,
            runtime: Some(runtime),
        }
    }
}

impl<F: Fn(usize, Status) + Send + Sync + 'static> Link for EtherCrab<F> {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        #[cfg(feature = "tokio")]
        {
            let runtime = if let Some(runtime) = self.runtime.take() {
                runtime
            } else {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_time()
                    .build()
                    .map_err(|e| LinkError::new(format!("Failed to create Tokio runtime: {}", e)))?
            };
            runtime.block_on(<Self as AsyncLink>::open(self, geometry))?;
            self.runtime = Some(runtime);
        }
        #[cfg(not(feature = "tokio"))]
        {
            crate::inner::executor::block_on(<Self as AsyncLink>::open(self, geometry))?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        #[cfg(feature = "tokio")]
        {
            let runtime = if let Some(runtime) = self.runtime.take() {
                runtime
            } else {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_time()
                    .build()
                    .map_err(|e| LinkError::new(format!("Failed to create Tokio runtime: {}", e)))?
            };
            runtime.block_on(<Self as AsyncLink>::close(self))?;
            self.runtime = Some(runtime);
        }
        #[cfg(not(feature = "tokio"))]
        {
            crate::inner::executor::block_on(<Self as AsyncLink>::close(self))?;
        }
        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.alloc_tx_buffer()
        } else {
            Err(LinkError::closed())
        }
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.send(tx)
        } else {
            Err(LinkError::closed())
        }
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.receive(rx);
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    fn is_open(&self) -> bool {
        self.inner.is_some()
    }
}

impl<F: Fn(usize, Status) + Send + Sync + 'static> AsyncLink for EtherCrab<F> {
    async fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        if let Some((err_handler, option)) = self.option.take() {
            self.inner = Some(EtherCrabHandler::open(err_handler, geometry, option).await?);
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        if let Some(mut inner) = self.inner.take() {
            inner.close().await?;
        }
        Ok(())
    }

    async fn update(&mut self, _: &Geometry) -> Result<(), LinkError> {
        Ok(())
    }

    async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        <Self as Link>::send(self, tx)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        <Self as Link>::receive(self, rx)
    }

    fn is_open(&self) -> bool {
        <Self as Link>::is_open(self)
    }

    async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        <Self as Link>::alloc_tx_buffer(self)
    }

    fn ensure_is_open(&self) -> Result<(), LinkError> {
        <Self as Link>::ensure_is_open(self)
    }
}
