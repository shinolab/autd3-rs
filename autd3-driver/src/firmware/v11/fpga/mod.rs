mod params;

pub use params::*;

pub use crate::firmware::v10::fpga::FPGAState;

#[must_use]
pub(crate) const fn ec_time_to_sys_time(time: &autd3_core::ethercat::DcSysTime) -> u64 {
    (time.sys_time() / 3125) << 6
}
