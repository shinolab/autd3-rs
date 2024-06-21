use std::net::SocketAddr;

use autd3_driver::{
    defined::{Freq, FREQ_40K},
    derive::*,
    geometry::{Device, Geometry, IntoDevice},
};

use crate::traits::*;

pub struct LightweightClient {
    client: crate::pb::ecat_light_client::EcatLightClient<tonic::transport::Channel>,
    geometry: Geometry,
}

#[derive(Builder)]
pub struct LightweightClientBuilder {
    devices: Vec<Device>,
    #[getset]
    ultrasound_freq: Freq<u32>,
}

impl LightweightClientBuilder {
    fn new<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> Self {
        Self {
            devices: iter
                .into_iter()
                .enumerate()
                .map(|(i, d)| d.into_device(i))
                .collect(),
            ultrasound_freq: FREQ_40K,
        }
    }

    pub async fn open(
        self,
        addr: SocketAddr,
    ) -> Result<LightweightClient, crate::error::AUTDProtoBufError> {
        LightweightClient::open_impl(Geometry::new(self.devices), addr).await
    }
}

impl LightweightClient {
    pub fn builder<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> LightweightClientBuilder {
        LightweightClientBuilder::new(iter)
    }

    async fn open_impl(
        geometry: Geometry,
        addr: SocketAddr,
    ) -> Result<Self, crate::error::AUTDProtoBufError> {
        let mut client =
            crate::pb::ecat_light_client::EcatLightClient::connect(format!("http://{}", addr))
                .await?;
        let res = client
            .config_geomety(geometry.to_msg(None))
            .await?
            .into_inner();
        if !res.success {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(Self { client, geometry })
    }

    pub async fn firmware_version(
        &mut self,
    ) -> Result<
        Vec<autd3_driver::firmware::version::FirmwareVersion>,
        crate::error::AUTDProtoBufError,
    > {
        let res = self
            .client
            .firmware_version(tonic::Request::new(
                crate::pb::FirmwareVersionRequestLightweight {},
            ))
            .await?
            .into_inner();
        if !res.success {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Vec::from_msg(&res)
    }

    pub async fn send(
        &mut self,
        datagram: impl ToMessage<Message = crate::pb::Datagram>,
    ) -> Result<bool, crate::error::AUTDProtoBufError> {
        let res = self
            .client
            .send(tonic::Request::new(datagram.to_msg(Some(&self.geometry))))
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(res.success)
    }

    pub async fn close(mut self) -> Result<(), crate::error::AUTDProtoBufError> {
        let res = self
            .client
            .close(crate::pb::CloseRequestLightweight {})
            .await?
            .into_inner();
        if !res.success {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(())
    }
}
