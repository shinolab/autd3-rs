use std::net::SocketAddr;

use autd3_driver::{
    derive::*,
    geometry::{Device, Geometry, IntoDevice},
};

use crate::{traits::*, OpenRequestLightweight};

pub struct LightweightClient {
    client: crate::pb::ecat_light_client::EcatLightClient<tonic::transport::Channel>,
    geometry: Geometry,
}

#[derive(Builder)]
pub struct LightweightClientBuilder {
    devices: Vec<Device>,
    #[get]
    #[set]
    default_parallel_threshold: usize,
    #[set]
    #[get]
    default_timeout: std::time::Duration,
    #[get]
    #[set]
    send_interval: std::time::Duration,
    #[get]
    #[set]
    receive_interval: std::time::Duration,
    #[cfg(target_os = "windows")]
    #[get]
    #[set]
    timer_resolution: u32,
}

impl LightweightClientBuilder {
    fn new<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> Self {
        Self {
            devices: iter
                .into_iter()
                .enumerate()
                .map(|(i, d)| d.into_device(i as _))
                .collect(),
            default_parallel_threshold: 4,
            default_timeout: std::time::Duration::from_millis(20),
            send_interval: std::time::Duration::from_millis(1),
            receive_interval: std::time::Duration::from_millis(1),
            #[cfg(target_os = "windows")]
            timer_resolution: 1,
        }
    }

    pub async fn open(
        self,
        addr: SocketAddr,
    ) -> Result<LightweightClient, crate::error::AUTDProtoBufError> {
        LightweightClient::open_impl(self, addr).await
    }
}

impl LightweightClient {
    pub fn builder<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> LightweightClientBuilder {
        LightweightClientBuilder::new(iter)
    }

    async fn open_impl(
        builder: LightweightClientBuilder,
        addr: SocketAddr,
    ) -> Result<Self, crate::error::AUTDProtoBufError> {
        let conn = tonic::transport::Endpoint::new(format!("http://{}", addr))?
            .connect()
            .await?;
        let mut client = crate::pb::ecat_light_client::EcatLightClient::new(conn);
        let geometry = Geometry::new(builder.devices, builder.default_parallel_threshold as _);
        let res = client
            .open(OpenRequestLightweight {
                geometry: Some(geometry.to_msg(None)),
                default_timeout: builder.default_timeout.as_nanos() as _,
                send_interval: builder.send_interval.as_nanos() as _,
                receive_interval: builder.receive_interval.as_nanos() as _,
            })
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
