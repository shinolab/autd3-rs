use autd3_core::link::{Link, LinkError, RxMessage, TxMessage};

use crate::{
    Status,
    inner::{EtherCrabHandler, EtherCrabOptionFull},
};

/// A [`Link`] using [EtherCrab](https://github.com/ethercrab-rs/ethercrab).
pub struct EtherCrab<F: Fn(usize, Status) + Send + Sync + 'static> {
    option: Option<(F, EtherCrabOptionFull)>,
    inner: Option<EtherCrabHandler>,
    #[cfg(feature = "tokio")]
    runtime: tokio::runtime::Runtime,
}

impl<F: Fn(usize, Status) + Send + Sync + 'static> EtherCrab<F> {
    /// Creates a new [`EtherCrab`]
    #[cfg(not(feature = "tokio"))]
    pub fn new(err_handler: F, option: impl Into<EtherCrabOptionFull>) -> Self {
        Self {
            option: Some((err_handler, option.into())),
            inner: None,
        }
    }

    /// Creates a new [`EtherCrab`]
    #[cfg(feature = "tokio")]
    pub fn new(err_handler: F, option: impl Into<EtherCrabOptionFull>) -> Self {
        Self::with_runtime(
            err_handler,
            option,
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
    }

    /// Creates a new [`EtherCrab`] with the given runtime.
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    #[cfg(feature = "tokio")]
    pub fn with_runtime(
        err_handler: F,
        option: impl Into<EtherCrabOptionFull>,
        runtime: tokio::runtime::Runtime,
    ) -> Self {
        Self {
            option: Some((err_handler, option.into())),
            inner: None,
            runtime,
        }
    }
}

impl<F: Fn(usize, Status) + Send + Sync + 'static> Link for EtherCrab<F> {
    fn open(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        if let Some((err_handler, option)) = self.option.take() {
            #[cfg(feature = "tokio")]
            let inner =
                self.runtime
                    .block_on(EtherCrabHandler::open(err_handler, &geometry, option))?;
            #[cfg(not(feature = "tokio"))]
            let inner = crate::inner::executor::block_on(EtherCrabHandler::open(
                err_handler,
                geometry,
                option,
            ))?;
            self.inner = Some(inner);
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        if let Some(mut inner) = self.inner.take() {
            #[cfg(feature = "tokio")]
            self.runtime.block_on(inner.close())?;
            #[cfg(not(feature = "tokio"))]
            crate::inner::executor::block_on(inner.close())?;
        }
        Ok(())
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.send(tx)?;
            Ok(())
        } else {
            Err(LinkError::new("Link is closed"))
        }
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.receive(rx);
            Ok(())
        } else {
            Err(LinkError::new("Link is closed"))
        }
    }

    fn is_open(&self) -> bool {
        self.inner.is_some()
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner
                .alloc_tx_buffer()
                .map_err(|e| LinkError::new(format!("Failed to allocate TX buffer: {}", e)))
        } else {
            Err(LinkError::closed())
        }
    }
}
