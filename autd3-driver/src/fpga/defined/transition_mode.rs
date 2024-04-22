const ECAT_DC_SYS_TIME_BASE: time::OffsetDateTime = time::macros::datetime!(2000-01-01 0:00 UTC);

#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum TransitionMode {
    SyncIdx,
    SysTime(time::OffsetDateTime),
    GPIO,
    Ext,
}

impl TransitionMode {
    pub(crate) fn mode(&self) -> u8 {
        match self {
            TransitionMode::SyncIdx => 0x00,
            TransitionMode::SysTime(_) => 0x01,
            TransitionMode::GPIO => 0x02,
            TransitionMode::Ext => 0xF0,
        }
    }

    pub(crate) fn value(&self) -> u64 {
        match self {
            TransitionMode::SyncIdx | TransitionMode::GPIO | TransitionMode::Ext => 0,
            TransitionMode::SysTime(time) => {
                (*time - ECAT_DC_SYS_TIME_BASE).whole_nanoseconds() as u64
            }
        }
    }
}
