use std::net::SocketAddr;

use autd3_core::geometry::{Device, Geometry, IntoDevice};

use derive_more::Deref;

use crate::{OpenRequestLightweight, traits::*};

#[derive(Deref)]
pub struct LightweightClient {
    client: crate::pb::ecat_light_client::EcatLightClient<tonic::transport::Channel>,
    #[deref]
    geometry: Geometry,
}

impl LightweightClient {
    pub async fn open<D: IntoDevice, F: IntoIterator<Item = D>>(
        devices: F,
        addr: SocketAddr,
    ) -> Result<Self, crate::error::AUTDProtoBufError> {
        LightweightClient::open_impl(
            devices
                .into_iter()
                .enumerate()
                .map(|(i, d)| d.into_device(i as _))
                .collect(),
            addr,
        )
        .await
    }

    async fn open_impl(
        devices: Vec<Device>,
        addr: SocketAddr,
    ) -> Result<Self, crate::error::AUTDProtoBufError> {
        let conn = tonic::transport::Endpoint::new(format!("http://{}", addr))?
            .connect()
            .await?;
        let mut client = crate::pb::ecat_light_client::EcatLightClient::new(conn);
        let geometry = Geometry::new(devices);
        let res = client
            .open(OpenRequestLightweight {
                geometry: Some(geometry.to_msg(None)?),
            })
            .await?
            .into_inner();
        if res.err {
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
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Vec::from_msg(res)
    }

    pub async fn send(
        &mut self,
        datagram: impl ToMessage<Message = crate::pb::Datagram>,
    ) -> Result<(), crate::error::AUTDProtoBufError> {
        let res = self
            .client
            .send(tonic::Request::new(datagram.to_msg(Some(&self.geometry))?))
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(())
    }

    pub async fn close(mut self) -> Result<(), crate::error::AUTDProtoBufError> {
        let res = self
            .client
            .close(crate::pb::CloseRequestLightweight {})
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(())
    }
}
