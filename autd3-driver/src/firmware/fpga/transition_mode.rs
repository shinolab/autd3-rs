use crate::ethercat::DcSysTime;

use super::GPIOIn;

pub const TRANSITION_MODE_SYNC_IDX: u8 = 0x00;
pub const TRANSITION_MODE_SYS_TIME: u8 = 0x01;
pub const TRANSITION_MODE_GPIO: u8 = 0x02;
pub const TRANSITION_MODE_EXT: u8 = 0xF0;
pub const TRANSITION_MODE_NONE: u8 = 0xFE;
pub const TRANSITION_MODE_IMMIDIATE: u8 = 0xFF;

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransitionMode {
    SyncIdx,
    SysTime(DcSysTime),
    GPIO(GPIOIn),
    Ext,
    Immidiate,
}

impl TransitionMode {
    pub fn mode(&self) -> u8 {
        match self {
            TransitionMode::SyncIdx => TRANSITION_MODE_SYNC_IDX,
            TransitionMode::SysTime(_) => TRANSITION_MODE_SYS_TIME,
            TransitionMode::GPIO(_) => TRANSITION_MODE_GPIO,
            TransitionMode::Ext => TRANSITION_MODE_EXT,
            TransitionMode::Immidiate => TRANSITION_MODE_IMMIDIATE,
        }
    }

    pub fn value(&self) -> u64 {
        match self {
            TransitionMode::SyncIdx | TransitionMode::Ext | TransitionMode::Immidiate => 0,
            TransitionMode::GPIO(gpio) => *gpio as u64,
            TransitionMode::SysTime(time) => time.sys_time(),
        }
    }
}
