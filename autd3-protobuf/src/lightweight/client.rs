use std::net::SocketAddr;

use autd3_driver::{
    defined::{Freq, FREQ_40K},
    geometry::{Device, Geometry, IntoDevice},
};

use crate::traits::*;

/// Client of AUTD with lightweight mode
pub struct LightweightClient {
    client: crate::pb::ecat_light_client::EcatLightClient<tonic::transport::Channel>,
    geometry: Geometry,
}

pub struct LightweightClientBuilder {
    devices: Vec<Device>,
    ultrasound_freq: Freq<u32>,
}

impl Default for LightweightClientBuilder {
    fn default() -> Self {
        Self::new_with_ultrasound_freq(FREQ_40K)
    }
}

impl LightweightClientBuilder {
    const fn new_with_ultrasound_freq(ultrasound_freq: Freq<u32>) -> Self {
        Self {
            devices: vec![],
            ultrasound_freq,
        }
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
        LightweightClient::open_impl(Geometry::new(self.devices, self.ultrasound_freq), addr).await
    }
}

impl LightweightClient {
    /// Create Client builder
    pub const fn builder() -> LightweightClientBuilder {
        Self::builder_with_ultrasound_freq(FREQ_40K)
    }

    /// Create Client builder
    pub const fn builder_with_ultrasound_freq(freq: Freq<u32>) -> LightweightClientBuilder {
        LightweightClientBuilder::new_with_ultrasound_freq(freq)
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
    /// * `Ok(Vec<FirmwareVersion>)` - List of firmware information
    ///
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
