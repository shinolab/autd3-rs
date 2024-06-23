use crate::geometry::Transducer;

use derive_more::Display;

#[non_exhaustive]
#[derive(Clone, Debug, Display)]
pub enum DebugType<'a> {
    None,
    BaseSignal,
    Thermo,
    ForceFan,
    Sync,
    ModSegment,
    #[display(fmt = "ModIdx({})", _0)]
    ModIdx(u16),
    StmSegment,
    #[display(fmt = "StmIdx({})", _0)]
    StmIdx(u16),
    IsStmMode,
    #[display(fmt = "PwmOut({})", "_0.idx()")]
    PwmOut(&'a Transducer),
    #[display(fmt = "Direct({})", _0)]
    Direct(bool),
}

impl DebugType<'_> {
    pub const fn ty(&self) -> u8 {
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

    pub const fn value(&self) -> u16 {
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

#[cfg(test)]
mod tests {
    use crate::geometry::Vector3;

    use super::*;

    #[test]
    fn display() {
        assert_eq!("None", DebugType::None.to_string());
        assert_eq!("BaseSignal", DebugType::BaseSignal.to_string());
        assert_eq!("Thermo", DebugType::Thermo.to_string());
        assert_eq!("ForceFan", DebugType::ForceFan.to_string());
        assert_eq!("Sync", DebugType::Sync.to_string());
        assert_eq!("ModSegment", DebugType::ModSegment.to_string());
        assert_eq!("ModIdx(1)", DebugType::ModIdx(1).to_string());
        assert_eq!("StmSegment", DebugType::StmSegment.to_string());
        assert_eq!("StmIdx(1)", DebugType::StmIdx(1).to_string());
        assert_eq!("IsStmMode", DebugType::IsStmMode.to_string());
        assert_eq!(
            "PwmOut(1)",
            DebugType::PwmOut(&Transducer::new(1, Vector3::new(0.0, 0.0, 0.0))).to_string()
        );
        assert_eq!("Direct(true)", DebugType::Direct(true).to_string());
    }
}
