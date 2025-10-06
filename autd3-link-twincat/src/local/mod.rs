use libloading as lib;

use std::ffi::c_void;

use lib::Library;

use autd3_core::{
    geometry::Geometry,
    link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage},
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

/// A [`Link`] using TwinCAT3.
///
/// To use this link, you need to install TwinCAT3 and run [`TwinCATAUTDServer`] before.
///
/// [`TwinCATAUTDServer`]: https://github.com/shinolab/autd3-server
pub struct TwinCAT {
    port: i32,
    send_addr: AmsAddr,
    buffer_pool: TxBufferPoolSync,
    dll: Library,
}

impl TwinCAT {
    /// Creates a new [`TwinCAT`].
    pub fn new() -> Result<TwinCAT, LinkError> {
        Ok(TwinCAT {
            port: 0,
            send_addr: AmsAddr {
                net_id: AmsNetId { b: [0; 6] },
                port: 0,
            },
            buffer_pool: TxBufferPoolSync::default(),
            dll: unsafe { lib::Library::new("TcAdsDll") }.map_err(|_| AdsError::DllNotFound)?,
        })
    }
}

impl Link for TwinCAT {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        let port = unsafe {
            self.dll
                .get::<unsafe extern "C" fn() -> i32>(b"AdsPortOpenEx")
                .map_err(|_| AdsError::FunctionNotFound("AdsPortOpenEx".to_owned()))?()
        };
        if port == 0 {
            return Err(AdsError::OpenPort.into());
        }

        let mut ams_addr: AmsAddr = unsafe { std::mem::zeroed() };
        let n_err = unsafe {
            self.dll
                .get::<unsafe extern "C" fn(i32, *mut AmsAddr) -> i32>(b"AdsGetLocalAddressEx")
                .map_err(|_| AdsError::FunctionNotFound("AdsGetLocalAddressEx".to_owned()))?(
                port,
                &mut ams_addr as *mut _,
            )
        };
        if n_err != 0 {
            return Err(AdsError::GetLocalAddress(n_err).into());
        }

        self.port = port;
        self.send_addr = AmsAddr {
            net_id: ams_addr.net_id,
            port: PORT,
        };

        self.buffer_pool.init(geometry);

        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        unsafe {
            self.dll
                .get::<unsafe extern "C" fn(i32) -> i32>(b"AdsPortCloseEx")
                .map_err(|_| AdsError::FunctionNotFound("AdsPortCloseEx".to_owned()))?(
                self.port
            );
        }
        self.port = 0;
        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        Ok(self.buffer_pool.borrow())
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        unsafe {
            let n_err = self.dll.get::<unsafe extern "C" fn(
                i32,
                *const AmsAddr,
                u32,
                u32,
                u32,
                *const c_void,
            ) -> i32>(b"AdsSyncWriteReqEx").map_err(|_|
                AdsError::FunctionNotFound(
                    "AdsSyncWriteReqEx".to_owned(),
                )
            )?
            (
                    self.port,
                    &raw const self.send_addr,
                    INDEX_GROUP,
                    INDEX_OFFSET_BASE,
                    (tx.len() * std::mem::size_of::<TxMessage>()) as _,
                    tx.as_ptr() as _,
            );

            self.buffer_pool.return_buffer(tx);

            if n_err > 0 {
                Err(AdsError::SendData(n_err).into())
            } else {
                Ok(())
            }
        }
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        let mut read_bytes: u32 = 0;
        unsafe {
            let n_err = self
                .dll
                .get::<unsafe extern "C" fn(
                    i32,
                    *const AmsAddr,
                    u32,
                    u32,
                    u32,
                    *mut c_void,
                    *mut u32,
                ) -> i32>(b"AdsSyncReadReqEx2")
                .map_err(|_| AdsError::FunctionNotFound("AdsSyncReadReqEx2".to_owned()))?(
                self.port,
                &raw const self.send_addr,
                INDEX_GROUP,
                INDEX_OFFSET_BASE_READ,
                std::mem::size_of_val(rx) as _,
                rx.as_mut_ptr() as *mut c_void,
                &mut read_bytes as *mut u32,
            );

            if n_err > 0 {
                Err(AdsError::ReadData(n_err).into())
            } else {
                Ok(())
            }
        }
    }

    fn is_open(&self) -> bool {
        self.port > 0
    }
}

#[cfg(feature = "async")]
use autd3_core::link::AsyncLink;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl AsyncLink for TwinCAT {
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
