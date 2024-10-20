use std::ffi::{c_long, CString};

use itertools::Itertools;

use zerocopy::IntoBytes;

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    link::{Link, LinkBuilder},
};

use crate::{error::AdsError, remote::native_methods::*};

const INDEX_GROUP: u32 = 0x0304_0030;
const INDEX_OFFSET_BASE: u32 = 0x8100_0000;
const INDEX_OFFSET_BASE_READ: u32 = 0x8000_0000;
const PORT: u16 = 301;

pub struct RemoteTwinCAT {
    port: c_long,
    net_id: AmsNetId,
}

#[derive(Builder, Debug)]
pub struct RemoteTwinCATBuilder {
    #[get(ref)]
    server_ams_net_id: String,
    #[get(ref)]
    #[set(into)]
    server_ip: String,
    #[get(ref)]
    #[set(into)]
    client_ams_net_id: String,
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for RemoteTwinCATBuilder {
    type L = RemoteTwinCAT;

    #[tracing::instrument(level = "debug", skip(_geometry))]
    async fn open(self, _geometry: &Geometry) -> Result<Self::L, AUTDInternalError> {
        tracing::info!("Connecting to TwinCAT3");

        let RemoteTwinCATBuilder {
            server_ams_net_id,
            server_ip,
            client_ams_net_id,
        } = self;

        let octets = server_ams_net_id
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
            server_ip
        };
        tracing::info!("Server IP: {}", ip);

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
            tracing::info!("Setting local AMS Net ID: {:?}", local_addr);
            unsafe {
                AdsCSetLocalAddress(local_addr);
            }
        }

        let net_id = AmsNetId {
            b: [
                octets[0], octets[1], octets[2], octets[3], octets[4], octets[5],
            ],
        };

        tracing::info!("Setting remote AMS Net ID: {:?}", net_id);
        let ip = CString::new(ip.clone()).map_err(|_| AdsError::InvalidIp(ip.clone()))?;
        let res = unsafe { AdsCAddRoute(net_id, ip.as_c_str().as_ptr()) };
        if res != 0 {
            return Err(AdsError::AmsAddRoute(res as _).into());
        }

        let port = unsafe { AdsCPortOpenEx() };

        if port == 0 {
            return Err(AdsError::OpenPort.into());
        }

        Ok(Self::L { port, net_id })
    }
}

impl RemoteTwinCAT {
    pub fn builder(server_ams_net_id: impl Into<String>) -> RemoteTwinCATBuilder {
        RemoteTwinCATBuilder {
            server_ams_net_id: server_ams_net_id.into(),
            server_ip: String::new(),
            client_ams_net_id: String::new(),
        }
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for RemoteTwinCAT {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
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

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
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

        if res == 0 {
            return Ok(true);
        }

        if res == ADSERR_DEVICE_INVALIDSIZE {
            return Err(AdsError::DeviceInvalidSize.into());
        }

        Err(AdsError::SendData(res as _).into())
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
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
            return Ok(true);
        }

        Err(AdsError::ReadData(res as _).into())
    }

    fn is_open(&self) -> bool {
        self.port > 0
    }
}
