/*
 * File: twincat_link.rs
 * Project: src
 * Created Date: 27/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 09/11/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Shun Suzuki. All rights reserved.
 *
 */

use std::{
    ffi::{c_long, CString},
    time::Duration,
};

use autd3_derive::Link;
use itertools::Itertools;

use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    error::AUTDInternalError,
    geometry::Geometry,
    link::{LinkSync, LinkSyncBuilder},
};

use crate::{error::AdsError, remote::native_methods::*};

const INDEX_GROUP: u32 = 0x0304_0030;
const INDEX_OFFSET_BASE: u32 = 0x8100_0000;
const INDEX_OFFSET_BASE_READ: u32 = 0x8000_0000;
const PORT: u16 = 301;

/// Link for remote TwinCAT3 server via [ADS](https://github.com/Beckhoff/ADS) library
#[derive(Link)]
pub struct RemoteTwinCAT {
    port: c_long,
    net_id: AmsNetId,
    timeout: Duration,
}

pub struct RemoteTwinCATBuilder {
    server_ams_net_id: String,
    server_ip: Option<String>,
    client_ams_net_id: Option<String>,
    timeout: Duration,
}

impl LinkSyncBuilder for RemoteTwinCATBuilder {
    type L = RemoteTwinCAT;

    fn open(self, _: &Geometry) -> Result<Self::L, AUTDInternalError> {
        let RemoteTwinCATBuilder {
            server_ams_net_id,
            mut server_ip,
            mut client_ams_net_id,
            timeout,
        } = self;

        let octets = server_ams_net_id
            .split('.')
            .map(|octet| octet.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| AUTDInternalError::from(AdsError::AmsNetIdParse))?;

        if octets.len() != 6 {
            return Err(AdsError::AmsNetIdParse.into());
        }

        let ip = if let Some(server_ip) = server_ip.take() {
            server_ip
        } else {
            octets[0..4].iter().map(|v| v.to_string()).join(".")
        };

        if let Some(client_ams_net_id) = client_ams_net_id.take() {
            let local_octets = client_ams_net_id
                .split('.')
                .map(|octet| octet.parse::<u8>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| AUTDInternalError::from(AdsError::AmsNetIdParse))?;
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

        let ip = CString::new(ip).unwrap();
        let res = unsafe { AdsCAddRoute(net_id, ip.as_c_str().as_ptr()) };
        if res != 0 {
            return Err(AdsError::AmsAddRoute(res as _).into());
        }

        let port = unsafe { AdsCPortOpenEx() };

        if port == 0 {
            return Err(AdsError::OpenPort.into());
        }

        Ok(Self::L {
            port,
            net_id,
            timeout,
        })
    }
}

impl RemoteTwinCATBuilder {
    /// Set server IP address
    pub fn with_server_ip<S: Into<String>>(mut self, server_ip: S) -> Self {
        self.server_ip = Some(server_ip.into());
        self
    }

    /// Set client AMS Net ID
    pub fn with_client_ams_net_id<S: Into<String>>(mut self, client_ams_net_id: S) -> Self {
        self.client_ams_net_id = Some(client_ams_net_id.into());
        self
    }

    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }
}

impl RemoteTwinCAT {
    pub fn builder<S: Into<String>>(server_ams_net_id: S) -> RemoteTwinCATBuilder {
        RemoteTwinCATBuilder {
            server_ams_net_id: server_ams_net_id.into(),
            server_ip: None,
            client_ams_net_id: None,
            timeout: Duration::from_millis(200),
        }
    }
}

impl LinkSync for RemoteTwinCAT {
    fn close(&mut self) -> Result<(), AUTDInternalError> {
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

    fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
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
                tx.all_data().len() as _,
                tx.all_data().as_ptr() as _,
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

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
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

    fn timeout(&self) -> Duration {
        self.timeout
    }
}
