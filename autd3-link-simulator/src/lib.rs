#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a link for [`AUTD3 Simulator`].
//!
//! [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server

use autd3_protobuf::*;

use std::net::SocketAddr;

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxMessage},
    link::{Link, LinkBuilder},
};

/// A [`Link`] for [`AUTD3 Simulator`].
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

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for SimulatorBuilder {
    type L = Simulator;

    #[tracing::instrument(level = "debug", skip(geometry))]
    async fn open(
        self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<Self::L, AUTDDriverError> {
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

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for Simulator {
    async fn close(&mut self) -> Result<(), AUTDDriverError> {
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

    async fn update(
        &mut self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<(), AUTDDriverError> {
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

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError> {
        let res = self
            .client
            .send_data(tx.to_msg(None))
            .await
            .map_err(AUTDProtoBufError::from)?;

        Ok(res.into_inner().success)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError> {
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
