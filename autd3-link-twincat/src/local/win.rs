use std::{
    ffi::{CStr, c_char, c_void},
    mem::MaybeUninit,
};

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

#[allow(clippy::upper_case_acronyms)]
type HMODULE = std::os::windows::raw::HANDLE;
#[allow(clippy::upper_case_acronyms)]
type FARPROC = *mut c_void;

unsafe extern "system" {
    fn LoadLibraryA(lp_lib_file_name: *const c_char) -> HMODULE;
    fn GetProcAddress(h_module: HMODULE, lp_proc_name: *const c_char) -> FARPROC;
    fn FreeLibrary(h_lib_module: HMODULE) -> i32;
}

/// A [`Link`] using TwinCAT3.
///
/// To use this link, you need to install TwinCAT3 and run [`TwinCATAUTDServer`] before.
///
/// [`TwinCATAUTDServer`]: https://github.com/shinolab/autd3-server
pub struct TwinCAT {
    port: i32,
    send_addr: AmsAddr,
    buffer_pool: TxBufferPoolSync,
    dll_handle: HMODULE,
}

unsafe impl Send for TwinCAT {}

impl TwinCAT {
    /// Creates a new [`TwinCAT`].
    pub fn new() -> Result<TwinCAT, LinkError> {
        let dll_handle = unsafe { LoadLibraryA(c"TcAdsDll.dll".as_ptr()) };
        if dll_handle.is_null() {
            return Err(AdsError::DllNotFound.into());
        }

        Ok(TwinCAT {
            port: 0,
            send_addr: AmsAddr {
                net_id: AmsNetId { b: [0; 6] },
                port: 0,
            },
            buffer_pool: TxBufferPoolSync::default(),
            dll_handle,
        })
    }

    unsafe fn get_proc_address<F>(&self, name: &CStr) -> Result<F, LinkError> {
        let proc = unsafe { GetProcAddress(self.dll_handle, name.as_ptr()) };
        if proc.is_null() {
            return Err(AdsError::FunctionNotFound(name.to_string_lossy().into_owned()).into());
        }
        Ok(unsafe { std::mem::transmute_copy(&proc) })
    }
}

impl Link for TwinCAT {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        let port = unsafe {
            let ads_port_open_ex: unsafe extern "C" fn() -> i32 =
                self.get_proc_address(c"AdsPortOpenEx")?;
            ads_port_open_ex()
        };
        if port == 0 {
            return Err(AdsError::OpenPort.into());
        }

        let mut ams_addr: MaybeUninit<AmsAddr> = MaybeUninit::uninit();
        let n_err = unsafe {
            let ads_get_local_address_ex: unsafe extern "C" fn(i32, *mut AmsAddr) -> i32 =
                self.get_proc_address(c"AdsGetLocalAddressEx")?;
            ads_get_local_address_ex(port, ams_addr.as_mut_ptr())
        };
        if n_err != 0 {
            return Err(AdsError::GetLocalAddress(n_err).into());
        }

        let ams_addr = unsafe { ams_addr.assume_init() };
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
            let ads_port_close_ex: unsafe extern "C" fn(i32) -> i32 =
                self.get_proc_address(c"AdsPortCloseEx")?;
            ads_port_close_ex(self.port);
        }
        self.port = 0;
        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        Ok(self.buffer_pool.borrow())
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        unsafe {
            let ads_sync_write_req_ex: unsafe extern "C" fn(
                i32,
                *const AmsAddr,
                u32,
                u32,
                u32,
                *const c_void,
            ) -> i32 = self.get_proc_address(c"AdsSyncWriteReqEx")?;

            let n_err = ads_sync_write_req_ex(
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
            let ads_sync_read_req_ex2: unsafe extern "C" fn(
                i32,
                *const AmsAddr,
                u32,
                u32,
                u32,
                *mut c_void,
                *mut u32,
            ) -> i32 = self.get_proc_address(c"AdsSyncReadReqEx2")?;

            let n_err = ads_sync_read_req_ex2(
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

impl autd3_core::link::AsyncLink for TwinCAT {}

impl Drop for TwinCAT {
    fn drop(&mut self) {
        if !self.dll_handle.is_null() {
            unsafe {
                FreeLibrary(self.dll_handle);
            }
            self.dll_handle = std::ptr::null_mut();
        }
    }
}
