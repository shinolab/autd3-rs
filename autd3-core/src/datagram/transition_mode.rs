use crate::ethercat::DcSysTime;

use super::fpga_gpio::GPIOIn;

pub(crate) const TRANSITION_MODE_SYNC_IDX: u8 = 0x00;
pub(crate) const TRANSITION_MODE_SYS_TIME: u8 = 0x01;
pub(crate) const TRANSITION_MODE_GPIO: u8 = 0x02;
pub(crate) const TRANSITION_MODE_EXT: u8 = 0xF0;
#[doc(hidden)]
pub const TRANSITION_MODE_NONE: u8 = 0xFE;
pub(crate) const TRANSITION_MODE_IMMEDIATE: u8 = 0xFF;

/// Transition mode of segment
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TransitionMode {
    /// Transition when the sampling index in the destination segment is 0.
    SyncIdx,
    /// Transition when the system time is the specified time.
    SysTime(DcSysTime),
    /// Transition when the specified GPIO pin is high.
    GPIO(GPIOIn),
    /// Transition to the next segment automatically when the data in the current segment is finished.
    Ext,
    /// Transition immediately.
    Immediate,
}

impl TransitionMode {
    #[doc(hidden)]
    pub const fn mode(self) -> u8 {
        match self {
            TransitionMode::SyncIdx => TRANSITION_MODE_SYNC_IDX,
            TransitionMode::SysTime(_) => TRANSITION_MODE_SYS_TIME,
            TransitionMode::GPIO(_) => TRANSITION_MODE_GPIO,
            TransitionMode::Ext => TRANSITION_MODE_EXT,
            TransitionMode::Immediate => TRANSITION_MODE_IMMEDIATE,
        }
    }

    #[doc(hidden)]
    pub const fn value(self) -> u64 {
        match self {
            TransitionMode::SyncIdx | TransitionMode::Ext | TransitionMode::Immediate => 0,
            TransitionMode::GPIO(gpio) => gpio as u64,
            TransitionMode::SysTime(time) => time.sys_time(),
        }
    }
}
