mod fpga_state;
mod stm_focus;

pub use fpga_state::*;
pub(crate) use stm_focus::*;

#[must_use]
pub(crate) const fn ec_time_to_sys_time(time: &autd3_core::ethercat::DcSysTime) -> u64 {
    (time.sys_time() / 3125) << 6
}
