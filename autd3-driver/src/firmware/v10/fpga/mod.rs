mod fpga_state;
mod params;
mod stm_focus;

pub use fpga_state::FPGAState;
pub use params::*;
pub(crate) use stm_focus::STMFocus;

#[must_use]
pub(crate) const fn ec_time_to_sys_time(time: crate::ethercat::DcSysTime) -> u64 {
    (time.sys_time() / 3125) << 5
}
