#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a link for [`AUTD3 Simulator`].
//!
//! [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server

use autd3_core::link::{AsyncLink, LinkError, RxMessage, TxBufferPoolSync, TxMessage};

use autd3_protobuf::*;

use std::net::SocketAddr;

struct SimulatorInner {
    client: simulator_client::SimulatorClient<tonic::transport::Channel>,
    last_geometry_version: usize,
    buffer_pool: TxBufferPoolSync,
}

impl SimulatorInner {
    async fn open(
        addr: &SocketAddr,
        geometry: &autd3_core::geometry::Geometry,
    ) -> Result<SimulatorInner, LinkError> {
        let conn = tonic::transport::Endpoint::new(format!("http://{addr}"))
            .map_err(AUTDProtoBufError::from)?
            .connect()
            .await
            .map_err(AUTDProtoBufError::from)?;
        let mut client = simulator_client::SimulatorClient::new(conn);

        client
            .config_geometry(Geometry::from(geometry))
            .await
            .map_err(|e| {
                AUTDProtoBufError::SendError(format!("Failed to initialize simulator: {e}"))
            })?;

        let mut buffer_pool = TxBufferPoolSync::default();
        buffer_pool.init(geometry);

        Ok(Self {
            client,
            last_geometry_version: geometry.version(),
            buffer_pool,
        })
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        self.client
            .close(CloseRequest {})
            .await
            .map_err(AUTDProtoBufError::from)?;

        Ok(())
    }

    async fn update(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        if self.last_geometry_version == geometry.version() {
            return Ok(());
        }
        self.last_geometry_version = geometry.version();
        self.client
            .update_geometry(Geometry::from(geometry))
            .await
            .map_err(|e| AUTDProtoBufError::SendError(format!("Failed to update geometry: {e}")))?;
        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Vec<TxMessage> {
        self.buffer_pool.borrow()
    }

    async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        let tx_data = TxRawData::from(tx.as_slice());
        self.buffer_pool.return_buffer(tx);
        self.client
            .send_data(tx_data)
            .await
            .map_err(AUTDProtoBufError::from)?;
        Ok(())
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
        let rx_ = Vec::<RxMessage>::from_msg(
            self.client
                .read_data(ReadRequest {})
                .await
                .map_err(AUTDProtoBufError::from)?
                .into_inner(),
        )?;
        if rx.len() == rx_.len() {
            rx.copy_from_slice(&rx_);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// A [`AsyncLink`] for [`AUTD3 Simulator`].
///
/// [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server
pub struct Simulator {
    num_devices: usize,
    addr: SocketAddr,
    inner: Option<SimulatorInner>,
    #[cfg(feature = "blocking")]
    runtime: Option<tokio::runtime::Runtime>,
}

impl Simulator {
    /// Creates a new [`Simulator`].
    #[must_use]
    pub const fn new(addr: SocketAddr) -> Simulator {
        Simulator {
            num_devices: 0,
            addr,
            inner: None,
            #[cfg(feature = "blocking")]
            runtime: None,
        }
    }
}

impl AsyncLink for Simulator {
    async fn open(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        self.inner = Some(SimulatorInner::open(&self.addr, geometry).await?);
        self.num_devices = geometry.len();
        Ok(())
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        if let Some(mut inner) = self.inner.take() {
            inner.close().await?;
        }
        Ok(())
    }

    async fn update(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.update(geometry).await?;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            Ok(inner.alloc_tx_buffer())
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
            inner.receive(rx).await?;
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
impl Link for Simulator {
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
            .map_or(Err(LinkError::closed()), |runtime| {
                runtime.block_on(async {
                    if let Some(mut inner) = self.inner.take() {
                        inner.close().await?;
                    }
                    Ok(())
                })
            })
    }

    fn update(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        self.runtime
            .as_ref()
            .map_or(Err(LinkError::closed()), |runtime| {
                runtime.block_on(async {
                    if let Some(inner) = self.inner.as_mut() {
                        inner.update(geometry).await?;
                    }
                    Ok(())
                })
            })
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        self.runtime
            .as_ref()
            .map_or(Err(LinkError::closed()), |runtime| {
                runtime.block_on(async {
                    if let Some(inner) = self.inner.as_mut() {
                        Ok(inner.alloc_tx_buffer())
                    } else {
                        Err(LinkError::closed())
                    }
                })
            })
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        self.runtime
            .as_ref()
            .map_or(Err(LinkError::closed()), |runtime| {
                runtime.block_on(async {
                    if let Some(inner) = self.inner.as_mut() {
                        inner.send(tx).await?;
                        Ok(())
                    } else {
                        Err(LinkError::closed())
                    }
                })
            })
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        self.runtime
            .as_ref()
            .map_or(Err(LinkError::closed()), |runtime| {
                runtime.block_on(async {
                    if let Some(inner) = self.inner.as_mut() {
                        inner.receive(rx).await?;
                        Ok(())
                    } else {
                        Err(LinkError::closed())
                    }
                })
            })
    }

    fn is_open(&self) -> bool {
        self.runtime.is_some() && self.inner.is_some()
    }
}
