#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a link for [`AUTD3 Simulator`].
//!
//! [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server

use autd3_core::link::{AsyncLink, AsyncLinkBuilder, LinkError, RxMessage, TxMessage};

use autd3_protobuf::*;

use std::net::SocketAddr;

/// A [`AsyncLink`] for [`AUTD3 Simulator`].
///
/// [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server
pub struct Simulator {
    client: simulator_client::SimulatorClient<tonic::transport::Channel>,
    is_open: bool,
    last_geometry_version: usize,
}

/// A builder for [`Simulator`].
#[derive(Builder, Debug)]
pub struct SimulatorBuilder {
    #[get]
    /// AUTD3 Simulator address.
    addr: SocketAddr,
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncLinkBuilder for SimulatorBuilder {
    type L = Simulator;

    #[tracing::instrument(level = "debug", skip(geometry))]
    async fn open(self, geometry: &autd3_core::geometry::Geometry) -> Result<Self::L, LinkError> {
        tracing::info!("Connecting to simulator@{}", self.addr);
        let conn = tonic::transport::Endpoint::new(format!("http://{}", self.addr))
            .map_err(AUTDProtoBufError::from)?
            .connect()
            .await
            .map_err(AUTDProtoBufError::from)?;
        let mut client = simulator_client::SimulatorClient::new(conn);

        client
            .config_geomety(geometry.to_msg(None))
            .await
            .map_err(|e| {
                tracing::error!("Failed to configure simulator geometry: {}", e);
                AUTDProtoBufError::SendError("Failed to initialize simulator".to_string())
            })?;

        Ok(Self::L {
            client,
            is_open: true,
            last_geometry_version: geometry.version(),
        })
    }
}

impl Simulator {
    /// Creates a new [`SimulatorBuilder`].
    pub const fn builder(addr: SocketAddr) -> SimulatorBuilder {
        SimulatorBuilder { addr }
    }
}

#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncLink for Simulator {
    async fn close(&mut self) -> Result<(), LinkError> {
        if !self.is_open {
            return Ok(());
        }
        self.is_open = false;

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
            .update_geomety(geometry.to_msg(None))
            .await
            .map_err(|e| {
                tracing::error!("Failed to update geometry: {}", e);
                AUTDProtoBufError::SendError("Failed to update geometry".to_string())
            })?;
        Ok(())
    }

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, LinkError> {
        let res = self
            .client
            .send_data(tx.to_msg(None))
            .await
            .map_err(AUTDProtoBufError::from)?;

        Ok(res.into_inner().success)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
        let rx_ = Vec::<RxMessage>::from_msg(
            &self
                .client
                .read_data(ReadRequest {})
                .await
                .map_err(AUTDProtoBufError::from)?
                .into_inner(),
        )?;
        if rx.len() == rx_.len() {
            rx.copy_from_slice(&rx_);
        }

        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

#[cfg(feature = "blocking")]
use autd3_core::link::{Link, LinkBuilder};

/// A [`Link`] for [`AUTD3 Simulator`].
///
/// [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
#[cfg(feature = "blocking")]
pub struct SimulatorBlocking {
    runtime: tokio::runtime::Runtime,
    inner: Simulator,
}

#[cfg(feature = "blocking")]
impl Link for SimulatorBlocking {
    fn close(&mut self) -> Result<(), LinkError> {
        self.runtime.block_on(self.inner.close())
    }

    fn update(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        self.runtime.block_on(self.inner.update(geometry))
    }

    fn send(&mut self, tx: &[TxMessage]) -> Result<bool, LinkError> {
        self.runtime.block_on(self.inner.send(tx))
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
        self.runtime.block_on(self.inner.receive(rx))
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    fn trace(&mut self, timeout: Option<std::time::Duration>, parallel_threshold: Option<usize>) {
        self.inner.trace(timeout, parallel_threshold)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
#[cfg(feature = "blocking")]
impl LinkBuilder for SimulatorBuilder {
    type L = SimulatorBlocking;

    fn open(self, geometry: &autd3_core::geometry::Geometry) -> Result<Self::L, LinkError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        let inner = runtime.block_on(<Self as AsyncLinkBuilder>::open(self, geometry))?;
        Ok(Self::L { runtime, inner })
    }
}
