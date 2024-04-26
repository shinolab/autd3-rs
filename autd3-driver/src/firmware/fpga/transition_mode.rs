use crate::ethercat::DcSysTime;

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TransitionMode {
    #[default]
    SyncIdx,
    SysTime(DcSysTime),
    GPIO,
    Ext,
}

impl TransitionMode {
    pub fn mode(&self) -> u8 {
        match self {
            TransitionMode::SyncIdx => 0x00,
            TransitionMode::SysTime(_) => 0x01,
            TransitionMode::GPIO => 0x02,
            TransitionMode::Ext => 0xF0,
        }
    }

    pub fn value(&self) -> u64 {
        match self {
            TransitionMode::SyncIdx | TransitionMode::GPIO | TransitionMode::Ext => 0,
            TransitionMode::SysTime(time) => time.sys_time(),
        }
    }
}
