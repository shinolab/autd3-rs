use crate::ethercat::DcSysTime;

use super::fpga_gpio::GPIOIn;

const TRANSITION_MODE_SYNC_IDX: u8 = 0x00;
const TRANSITION_MODE_SYS_TIME: u8 = 0x01;
const TRANSITION_MODE_GPIO: u8 = 0x02;
const TRANSITION_MODE_EXT: u8 = 0xF0;
const TRANSITION_MODE_NONE: u8 = 0xFE;
const TRANSITION_MODE_IMMEDIATE: u8 = 0xFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub struct TransitionModeParams {
    pub mode: u8,
    pub value: u64,
}

/// Transition mode between segments.
pub trait TransitionMode: Clone + Copy + Send + Sync {
    #[doc(hidden)]
    fn params(self) -> TransitionModeParams;
}

/// Transition when the sampling index in the destination segment is 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyncIdx;
impl TransitionMode for SyncIdx {
    fn params(self) -> TransitionModeParams {
        TransitionModeParams {
            mode: TRANSITION_MODE_SYNC_IDX,
            value: 0,
        }
    }
}

/// Transition when the system time is the specified time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SysTime(pub DcSysTime);
impl TransitionMode for SysTime {
    fn params(self) -> TransitionModeParams {
        TransitionModeParams {
            mode: TRANSITION_MODE_SYS_TIME,
            value: self.0.sys_time(),
        }
    }
}

/// Transition when the specified GPIO pin is high.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GPIO(pub GPIOIn);
impl TransitionMode for GPIO {
    fn params(self) -> TransitionModeParams {
        TransitionModeParams {
            mode: TRANSITION_MODE_GPIO,
            value: self.0 as u64,
        }
    }
}

/// Transition immediately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Immediate;
impl TransitionMode for Immediate {
    fn params(self) -> TransitionModeParams {
        TransitionModeParams {
            mode: TRANSITION_MODE_IMMEDIATE,
            value: 0,
        }
    }
}

/// Transition to the next segment automatically when the data in the current segment is finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ext;
impl TransitionMode for Ext {
    fn params(self) -> TransitionModeParams {
        TransitionModeParams {
            mode: TRANSITION_MODE_EXT,
            value: 0,
        }
    }
}

/// Transition later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Later;
impl TransitionMode for Later {
    fn params(self) -> TransitionModeParams {
        TransitionModeParams {
            mode: TRANSITION_MODE_NONE,
            value: 0,
        }
    }
}
