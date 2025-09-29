use autd3_core::{
    link::{AsyncLink, LinkError, RxMessage, TxMessage},
    sleep::Sleep,
};
use spin_sleep::SpinSleeper;

use crate::{
    Status,
    inner::{EtherCrabHandler, EtherCrabOptionFull},
};

/// A [`AsyncLink`] using [ethercrab].
pub struct EtherCrab<F: Fn(usize, Status) + Send + Sync + 'static, S: Sleep + Send + 'static> {
    option: Option<(F, EtherCrabOptionFull, S)>,
    inner: Option<EtherCrabHandler>,
    #[cfg(feature = "blocking")]
    runtime: Option<tokio::runtime::Runtime>,
}

impl<F: Fn(usize, Status) + Send + Sync + 'static> EtherCrab<F, SpinSleeper> {
    /// Creates a new [`EtherCrab`].
    pub fn new(err_handler: F, option: impl Into<EtherCrabOptionFull>) -> Self {
        Self::with_sleeper(err_handler, option, SpinSleeper::default())
    }
}

impl<F: Fn(usize, Status) + Send + Sync + 'static, S: Sleep + Send + 'static> EtherCrab<F, S> {
    /// Creates a new [`EtherCrab`] with a sleeper.
    pub fn with_sleeper(
        err_handler: F,
        option: impl Into<EtherCrabOptionFull>,
        sleeper: S,
    ) -> Self {
        Self {
            option: Some((err_handler, option.into(), sleeper)),
            inner: None,
            #[cfg(feature = "blocking")]
            runtime: None,
        }
    }
}

impl<F: Fn(usize, Status) + Send + Sync + 'static, S: Sleep + Send + 'static> AsyncLink
    for EtherCrab<F, S>
{
    async fn open(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        if let Some((err_handler, option, sleeper)) = self.option.take() {
            self.inner =
                Some(EtherCrabHandler::open(err_handler, geometry, option, sleeper).await?);
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        if let Some(mut inner) = self.inner.take() {
            inner.close().await?;
        }
        Ok(())
    }

    async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner
                .alloc_tx_buffer()
                .await
                .map_err(|e| LinkError::new(format!("Failed to allocate TX buffer: {}", e)))
        } else {
            Err(LinkError::closed())
        }
    }

    async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.send(tx).await?;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.receive(rx).await;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    fn is_open(&self) -> bool {
        self.inner.is_some()
    }
}

#[cfg(feature = "blocking")]
use autd3_core::link::Link;

#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
#[cfg(feature = "blocking")]
impl<F: Fn(usize, Status) + Send + Sync + 'static, S: Sleep + Send + 'static> Link
    for EtherCrab<F, S>
{
    fn open(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        runtime.block_on(<Self as AsyncLink>::open(self, geometry))?;
        self.runtime = Some(runtime);
        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        self.runtime
            .as_ref()
            .map_or(Err(LinkError::new("Link is closed")), |runtime| {
                runtime.block_on(async {
                    if let Some(mut inner) = self.inner.take() {
                        inner.close().await?;
                    }
                    Ok(())
                })
            })
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        self.runtime
            .as_ref()
            .map_or(Err(LinkError::new("Link is closed")), |runtime| {
                runtime.block_on(async {
                    if let Some(inner) = self.inner.as_mut() {
                        inner.send(tx).await?;
                        Ok(())
                    } else {
                        Err(LinkError::new("Link is closed"))
                    }
                })
            })
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        self.runtime
            .as_ref()
            .map_or(Err(LinkError::new("Link is closed")), |runtime| {
                runtime.block_on(async {
                    if let Some(inner) = self.inner.as_mut() {
                        inner.receive(rx).await;
                        Ok(())
                    } else {
                        Err(LinkError::new("Link is closed"))
                    }
                })
            })
    }

    fn is_open(&self) -> bool {
        self.runtime.is_some() && self.inner.is_some()
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        self.runtime
            .as_ref()
            .map_or(Err(LinkError::closed()), |runtime| {
                runtime.block_on(async {
                    if let Some(inner) = self.inner.as_mut() {
                        inner.alloc_tx_buffer().await.map_err(|e| {
                            LinkError::new(format!("Failed to allocate TX buffer: {}", e))
                        })
                    } else {
                        Err(LinkError::closed())
                    }
                })
            })
    }
}
