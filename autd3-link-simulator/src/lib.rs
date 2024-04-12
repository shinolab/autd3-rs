use autd3_protobuf::*;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    derive::*,
    link::{Link, LinkBuilder},
};

/// Link for Simulator
pub struct Simulator {
    client: simulator_client::SimulatorClient<tonic::transport::Channel>,
    timeout: Duration,
    is_open: bool,
}

#[derive(Builder)]
pub struct SimulatorBuilder {
    #[get]
    port: u16,
    #[getset]
    server_ip: IpAddr,
    #[getset]
    timeout: Duration,
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for SimulatorBuilder {
    type L = Simulator;

    async fn open(
        self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<Self::L, AUTDInternalError> {
        let mut client = simulator_client::SimulatorClient::connect(format!(
            "http://{}",
            SocketAddr::new(self.server_ip, self.port)
        ))
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
        })
    }
}

impl Simulator {
    pub const fn builder(port: u16) -> SimulatorBuilder {
        SimulatorBuilder {
            server_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,
            timeout: Duration::from_millis(200),
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

        if let Some(rx_) = Vec::<RxMessage>::from_msg(
            &self
                .client
                .read_data(ReadRequest {})
                .await
                .map_err(AUTDProtoBufError::from)?
                .into_inner(),
        ) {
            if rx.len() == rx_.len() {
                rx.copy_from_slice(&rx_);
            }
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

impl Simulator {
    pub async fn update_geometry(
        &mut self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<(), AUTDInternalError> {
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
}
