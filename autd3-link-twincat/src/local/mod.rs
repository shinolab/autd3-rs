use libloading as lib;

use std::ffi::c_void;

use lib::Library;

use zerocopy::IntoBytes;

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxMessage},
    link::{Link, LinkBuilder},
};

#[repr(C)]
#[derive(Copy, Clone)]
struct AmsNetId {
    b: [u8; 6],
}

#[repr(C)]
struct AmsAddr {
    net_id: AmsNetId,
    port: u16,
}

use crate::error::AdsError;

const INDEX_GROUP: u32 = 0x0304_0030;
const INDEX_OFFSET_BASE: u32 = 0x8100_0000;
const INDEX_OFFSET_BASE_READ: u32 = 0x8000_0000;
const PORT: u16 = 301;

pub struct TwinCAT {
    port: i32,
    send_addr: AmsAddr,
    dll: Library,
}

#[derive(Builder)]
pub struct TwinCATBuilder {}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for TwinCATBuilder {
    type L = TwinCAT;

    async fn open(self, _: &Geometry) -> Result<Self::L, AUTDInternalError> {
        let dll = match unsafe { lib::Library::new("TcAdsDll") } {
            Ok(dll) => dll,
            Err(_) => {
                return Err(AUTDInternalError::LinkError(
                    "TcAdsDll not found. Please install TwinCAT3".to_owned(),
                ))
            }
        };

        let port = unsafe {
            match dll.get::<unsafe extern "C" fn() -> i32>(b"AdsPortOpenEx") {
                Ok(f) => f(),
                Err(_) => {
                    return Err(AUTDInternalError::LinkError(
                        "AdsPortOpenEx not found".to_owned(),
                    ))
                }
            }
        };
        if port == 0 {
            return Err(AdsError::OpenPort.into());
        }

        let mut ams_addr: AmsAddr = unsafe { std::mem::zeroed() };
        let n_err = unsafe {
            match dll.get::<unsafe extern "C" fn(i32, *mut AmsAddr) -> i32>(b"AdsGetLocalAddressEx")
            {
                Ok(f) => f(port, &mut ams_addr as *mut _),
                Err(_) => {
                    return Err(AUTDInternalError::LinkError(
                        "AdsGetLocalAddressEx not found".to_owned(),
                    ))
                }
            }
        };
        if n_err != 0 {
            return Err(AdsError::GetLocalAddress(n_err).into());
        }

        Ok(Self::L {
            port,
            send_addr: AmsAddr {
                net_id: ams_addr.net_id,
                port: PORT,
            },
            dll,
        })
    }
}

impl TwinCAT {
    pub const fn builder() -> TwinCATBuilder {
        TwinCATBuilder {}
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for TwinCAT {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        unsafe {
            match self
                .dll
                .get::<unsafe extern "C" fn(i32) -> i32>(b"AdsPortCloseEx")
            {
                Ok(f) => f(self.port),
                Err(_) => {
                    return Err(AUTDInternalError::LinkError(
                        "AdsPortCloseEx not found".to_owned(),
                    ))
                }
            };
        }
        self.port = 0;
        Ok(())
    }

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDInternalError> {
        unsafe {
            let n_err = match self.dll.get::<unsafe extern "C" fn(
                i32,
                *const AmsAddr,
                u32,
                u32,
                u32,
                *const c_void,
            ) -> i32>(b"AdsSyncWriteReqEx")
            {
                Ok(f) => f(
                    self.port,
                    &self.send_addr as *const _,
                    INDEX_GROUP,
                    INDEX_OFFSET_BASE,
                    tx.as_bytes().len() as _,
                    tx.as_ptr() as _,
                ),
                Err(_) => {
                    return Err(AUTDInternalError::LinkError(
                        "AdsSyncWriteReqEx not found".to_owned(),
                    ))
                }
            };

            if n_err > 0 {
                Err(AdsError::SendData(n_err).into())
            } else {
                Ok(true)
            }
        }
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        let mut read_bytes: u32 = 0;
        unsafe {
            let n_err = match self.dll.get::<unsafe extern "C" fn(
                i32,
                *const AmsAddr,
                u32,
                u32,
                u32,
                *mut c_void,
                *mut u32,
            ) -> i32>(b"AdsSyncReadReqEx2")
            {
                Ok(f) => f(
                    self.port,
                    &self.send_addr as *const _,
                    INDEX_GROUP,
                    INDEX_OFFSET_BASE_READ,
                    std::mem::size_of_val(rx) as _,
                    rx.as_mut_ptr() as *mut c_void,
                    &mut read_bytes as *mut u32,
                ),
                Err(_) => {
                    return Err(AUTDInternalError::LinkError(
                        "AdsSyncReadReqEx2 not found".to_owned(),
                    ))
                }
            };

            if n_err > 0 {
                Err(AdsError::ReadData(n_err).into())
            } else {
                Ok(true)
            }
        }
    }

    fn is_open(&self) -> bool {
        self.port > 0
    }
}
