use std::ffi::{CString, c_long};

use itertools::Itertools;

use zerocopy::IntoBytes;

use autd3_core::{
    geometry::Geometry,
    link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage},
};

use crate::{error::AdsError, remote::native_methods::*};

const INDEX_GROUP: u32 = 0x0304_0030;
const INDEX_OFFSET_BASE: u32 = 0x8100_0000;
const INDEX_OFFSET_BASE_READ: u32 = 0x8000_0000;
const PORT: u16 = 301;

/// A [`Link`] using TwinCAT3.
///
/// To use this link, you need to install TwinCAT3 and run [`TwinCATAUTDServer`] on server side.
///
/// [`TwinCATAUTDServer`]: https://github.com/shinolab/autd3-server
pub struct RemoteTwinCAT {
    server_ams_net_id: String,
    option: RemoteTwinCATOption,
    port: c_long,
    net_id: AmsNetId,
    buffer_pool: TxBufferPoolSync,
}

/// The option of [`RemoteTwinCAT`].
#[derive(Debug, Default)]
pub struct RemoteTwinCATOption {
    /// The IP address of the TwinCAT3 server. If empty, the first 4 octets of `server_ams_net_id` are used.
    pub server_ip: String,
    /// The AMS Net ID of the client.
    pub client_ams_net_id: String,
}

impl RemoteTwinCAT {
    /// Creates a new [`RemoteTwinCAT`].
    #[must_use]
    pub fn new(server_ams_net_id: impl Into<String>, option: RemoteTwinCATOption) -> RemoteTwinCAT {
        RemoteTwinCAT {
            server_ams_net_id: server_ams_net_id.into(),
            option,
            port: 0,
            net_id: AmsNetId { b: [0; 6] },
            buffer_pool: TxBufferPoolSync::default(),
        }
    }
}

impl Link for RemoteTwinCAT {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        let RemoteTwinCATOption {
            server_ip,
            client_ams_net_id,
        } = &self.option;

        let octets = self
            .server_ams_net_id
            .split('.')
            .map(|octet| octet.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| AdsError::AmsNetIdParse)?;

        if octets.len() != 6 {
            return Err(AdsError::AmsNetIdParse.into());
        }

        let ip = if server_ip.is_empty() {
            octets[0..4].iter().map(|v| v.to_string()).join(".")
        } else {
            server_ip.to_owned()
        };

        if !client_ams_net_id.is_empty() {
            let local_octets = client_ams_net_id
                .split('.')
                .map(|octet| octet.parse::<u8>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| AdsError::AmsNetIdParse)?;
            if local_octets.len() != 6 {
                return Err(AdsError::AmsNetIdParse.into());
            }

            let local_addr = AmsNetId {
                b: [
                    local_octets[0],
                    local_octets[1],
                    local_octets[2],
                    local_octets[3],
                    local_octets[4],
                    local_octets[5],
                ],
            };
            unsafe {
                AdsCSetLocalAddress(local_addr);
            }
        }

        let net_id = AmsNetId {
            b: [
                octets[0], octets[1], octets[2], octets[3], octets[4], octets[5],
            ],
        };

        let ip = CString::new(ip.clone()).map_err(|_| AdsError::InvalidIp(ip.clone()))?;
        let res = unsafe { AdsCAddRoute(net_id, ip.as_c_str().as_ptr()) };
        if res != 0 {
            return Err(AdsError::AmsAddRoute(res as _).into());
        }

        let port = unsafe { AdsCPortOpenEx() };

        if port == 0 {
            return Err(AdsError::OpenPort.into());
        }

        self.port = port;
        self.net_id = net_id;

        self.buffer_pool.init(geometry);

        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        if self.port == 0 {
            return Ok(());
        }

        unsafe {
            if AdsCPortCloseEx(self.port) != 0 {
                return Err(AdsError::ClosePort.into());
            }
        }

        self.port = 0;

        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        Ok(self.buffer_pool.borrow())
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        let addr = AmsAddr {
            net_id: self.net_id,
            port: PORT,
        };

        let res = unsafe {
            AdsCSyncWriteReqEx(
                self.port,
                &addr as _,
                INDEX_GROUP,
                INDEX_OFFSET_BASE,
                tx.as_bytes().len() as _,
                tx.as_ptr() as _,
            )
        };

        self.buffer_pool.return_buffer(tx);

        if res == 0 {
            return Ok(());
        }
        if res == ADSERR_DEVICE_INVALIDSIZE {
            return Err(AdsError::DeviceInvalidSize.into());
        }
        Err(AdsError::SendData(res as _).into())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        let addr = AmsAddr {
            net_id: self.net_id,
            port: PORT,
        };

        let mut receive_bytes: u32 = 0;
        let res = unsafe {
            AdsCSyncReadReqEx2(
                self.port,
                &addr as _,
                INDEX_GROUP,
                INDEX_OFFSET_BASE_READ,
                std::mem::size_of_val(rx) as _,
                rx.as_mut_ptr() as _,
                &mut receive_bytes as _,
            )
        };

        if res == 0 {
            return Ok(());
        }
        Err(AdsError::ReadData(res as _).into())
    }

    fn is_open(&self) -> bool {
        self.port > 0
    }
}

#[cfg(feature = "async")]
use autd3_core::link::AsyncLink;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncLink for RemoteTwinCAT {
    async fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        <Self as Link>::open(self, geometry)
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        <Self as Link>::close(self)
    }

    async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        <Self as Link>::alloc_tx_buffer(self)
    }

    async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        <Self as Link>::send(self, tx)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        <Self as Link>::receive(self, rx)
    }

    fn is_open(&self) -> bool {
        <Self as Link>::is_open(self)
    }
}
