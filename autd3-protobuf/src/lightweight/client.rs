use std::net::SocketAddr;

use autd3_driver::geometry::{Device, Geometry, IntoDevice};

use crate::traits::*;

/// Client of AUTD with lightweight mode
pub struct LightweightClient {
    client: crate::pb::ecat_light_client::EcatLightClient<tonic::transport::Channel>,
    geometry: Geometry,
}

pub struct LightweightClientBuilder {
    devices: Vec<Device>,
}

impl Default for LightweightClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LightweightClientBuilder {
    const fn new() -> Self {
        Self { devices: vec![] }
    }

    /// Add device
    pub fn add_device(mut self, dev: impl IntoDevice) -> Self {
        self.devices.push(dev.into_device(self.devices.len()));
        self
    }

    /// Open connection
    pub async fn open(
        self,
        addr: SocketAddr,
    ) -> Result<LightweightClient, crate::error::AUTDProtoBufError> {
        LightweightClient::open_impl(Geometry::new(self.devices), addr).await
    }
}

impl LightweightClient {
    /// Create Client builder
    pub const fn builder() -> LightweightClientBuilder {
        LightweightClientBuilder::new()
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

    /// Get firmware information
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<FirmwareInfo>)` - List of firmware information
    ///
    pub async fn firmware_infos(
        &mut self,
    ) -> Result<
        Vec<autd3_driver::firmware::firmware_version::FirmwareInfo>,
        crate::error::AUTDProtoBufError,
    > {
        let res = self
            .client
            .firmware_info(tonic::Request::new(
                crate::pb::FirmwareInfoRequestLightweight {},
            ))
            .await?
            .into_inner();
        if !res.success {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        match Vec::from_msg(&res) {
            Some(v) => Ok(v),
            None => Err(crate::error::AUTDProtoBufError::DataParseError),
        }
    }

    /// Send data to the devices
    ///
    /// # Arguments
    ///
    /// * `s` - Datagram
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - It is confirmed that the data has been successfully transmitted
    /// * `Ok(false)` - There are no errors, but it is unclear whether the data has been sent reliably or not
    ///
    pub async fn send(
        &mut self,
        datagram: impl ToMessage<Message = crate::pb::DatagramLightweight>,
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

    // Close connection
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
