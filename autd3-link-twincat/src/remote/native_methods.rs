use std::ffi::{c_char, c_long, c_void};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub(crate) struct AmsNetId {
    pub(crate) b: [u8; 6],
}

#[repr(C)]
pub(crate) struct AmsAddr {
    pub(crate) net_id: AmsNetId,
    pub(crate) port: u16,
}

const ERR_ADSERRS: c_long = 0x0700;
pub const ADSERR_DEVICE_INVALIDSIZE: c_long = 0x05 + ERR_ADSERRS;

#[link(name = "ads", kind = "static")]
unsafe extern "C" {
    pub fn AdsCSetLocalAddress(ams: AmsNetId);
    pub fn AdsCAddRoute(ams: AmsNetId, ip: *const c_char) -> c_long;
    pub fn AdsCPortOpenEx() -> c_long;
    pub fn AdsCPortCloseEx(port: c_long) -> c_long;
    pub fn AdsCSyncWriteReqEx(
        port: c_long,
        pAddr: *const AmsAddr,
        indexGroup: u32,
        indexOffset: u32,
        bufferLength: u32,
        buffer: *const c_void,
    ) -> c_long;
    pub fn AdsCSyncReadReqEx2(
        port: c_long,
        pAddr: *const AmsAddr,
        indexGroup: u32,
        indexOffset: u32,
        bufferLength: u32,
        buffer: *mut c_void,
        bytesRead: *mut u32,
    ) -> c_long;
}
