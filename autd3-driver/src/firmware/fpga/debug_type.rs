use crate::geometry::Transducer;

#[non_exhaustive]
#[derive(Clone, Debug)]
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

impl<'a> std::fmt::Display for DebugType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugType::None => write!(f, "None"),
            DebugType::BaseSignal => write!(f, "BaseSignal"),
            DebugType::Thermo => write!(f, "Thermo"),
            DebugType::ForceFan => write!(f, "ForceFan"),
            DebugType::Sync => write!(f, "Sync"),
            DebugType::ModSegment => write!(f, "ModSegment"),
            DebugType::ModIdx(idx) => write!(f, "ModIdx({})", idx),
            DebugType::StmSegment => write!(f, "StmSegment"),
            DebugType::StmIdx(idx) => write!(f, "StmIdx({})", idx),
            DebugType::IsStmMode => write!(f, "IsStmMode"),
            DebugType::PwmOut(tr) => write!(f, "PwmOut({})", tr.idx()),
            DebugType::Direct(v) => write!(f, "Direct({})", v),
        }
    }
}
