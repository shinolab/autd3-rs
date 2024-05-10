use crate::geometry::Transducer;

#[non_exhaustive]
#[derive(Clone)]
pub enum DebugType<'a> {
    None,
    BaseSignal,
    Thermo,
    ForceFan,
    Sync,
    ModSegment,
    ModIdx(u16),
    StmSegment,
    StmIdx(u16),
    IsStmMode,
    PwmOut(&'a Transducer),
    Direct(bool),
}

impl DebugType<'_> {
    pub fn ty(&self) -> u8 {
        match self {
            DebugType::None => 0x00,
            DebugType::BaseSignal => 0x01,
            DebugType::Thermo => 0x02,
            DebugType::ForceFan => 0x03,
            DebugType::Sync => 0x10,
            DebugType::ModSegment => 0x20,
            DebugType::ModIdx(_) => 0x21,
            DebugType::StmSegment => 0x50,
            DebugType::StmIdx(_) => 0x51,
            DebugType::IsStmMode => 0x52,
            DebugType::PwmOut(_) => 0xE0,
            DebugType::Direct(_) => 0xF0,
        }
    }

    pub fn value(&self) -> u16 {
        match self {
            DebugType::None
            | DebugType::BaseSignal
            | DebugType::Thermo
            | DebugType::ForceFan
            | DebugType::Sync
            | DebugType::ModSegment
            | DebugType::StmSegment
            | DebugType::IsStmMode => 0,
            DebugType::PwmOut(tr) => tr.idx() as _,
            DebugType::ModIdx(idx) => *idx,
            DebugType::StmIdx(idx) => *idx,
            DebugType::Direct(v) => *v as u16,
        }
    }
}
