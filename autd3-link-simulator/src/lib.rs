use autd3_protobuf::*;

use std::{net::SocketAddr, time::Duration};

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    link::{Link, LinkBuilder},
};

pub struct Simulator {
    client: simulator_client::SimulatorClient<tonic::transport::Channel>,
    timeout: Duration,
    is_open: bool,
    last_geometry_version: usize,
}

#[derive(Builder)]
pub struct SimulatorBuilder {
    #[get]
    addr: SocketAddr,
    #[get]
    #[set]
    timeout: Duration,
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for SimulatorBuilder {
    type L = Simulator;

    async fn open(
        self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<Self::L, AUTDInternalError> {
        let mut client =
            simulator_client::SimulatorClient::connect(format!("http://{}", self.addr))
                .await
                .map_err(|e| AUTDInternalError::from(AUTDProtoBufError::from(e)))?;

        if client.config_geomety(geometry.to_msg(None)).await.is_err() {
            return Err(
                AUTDProtoBufError::SendError("Failed to initialize simulator".to_string()).into(),
            );
        }

        Ok(Self::L {
            client,
            timeout: self.timeout,
            is_open: true,
            last_geometry_version: geometry.version(),
        })
    }
}

impl Simulator {
    pub const fn builder(addr: SocketAddr) -> SimulatorBuilder {
        SimulatorBuilder {
            addr,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for Simulator {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
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
    ) -> Result<(), AUTDInternalError> {
        if self.last_geometry_version == geometry.version() {
            return Ok(());
        }
        self.last_geometry_version = geometry.version();
        if self
            .client
            .update_geomety(geometry.to_msg(None))
            .await
            .is_err()
        {
            return Err(
                AUTDProtoBufError::SendError("Failed to update geometry".to_string()).into(),
            );
        }
        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        if !self.is_open {
            return Err(AUTDInternalError::LinkClosed);
        }

        let res = self
            .client
            .send_data(tx.to_msg(None))
            .await
            .map_err(AUTDProtoBufError::from)?;

        Ok(res.into_inner().success)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        if !self.is_open {
            return Err(AUTDInternalError::LinkClosed);
        }

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

    fn timeout(&self) -> Duration {
        self.timeout
    }
}
