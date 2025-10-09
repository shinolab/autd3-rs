use autd3_core::link::{Link, LinkError, RxMessage, TxMessage};

use crate::{
    Status,
    inner::{EtherCrabHandler, EtherCrabOptionFull},
};

/// A [`Link`] using [EtherCrab](https://github.com/ethercrab-rs/ethercrab).
pub struct EtherCrab<F: Fn(usize, Status) + Send + Sync + 'static> {
    option: Option<(F, EtherCrabOptionFull)>,
    inner: Option<EtherCrabHandler>,
    runtime: Option<tokio::runtime::Runtime>,
}

impl<F: Fn(usize, Status) + Send + Sync + 'static> EtherCrab<F> {
    /// Creates a new [`EtherCrab`]
    pub fn new(err_handler: F, option: impl Into<EtherCrabOptionFull>) -> Self {
        Self {
            option: Some((err_handler, option.into())),
            inner: None,
            runtime: None,
        }
    }
}

impl<F: Fn(usize, Status) + Send + Sync + 'static> Link for EtherCrab<F> {
    fn open(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        runtime.block_on(async {
            if let Some((err_handler, option)) = self.option.take() {
                self.inner = Some(EtherCrabHandler::open(err_handler, geometry, option).await?);
            }
            Result::<(), LinkError>::Ok(())
        })?;
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
