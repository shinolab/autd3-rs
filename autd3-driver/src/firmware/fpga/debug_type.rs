use crate::geometry::Transducer;

use derive_more::Debug;

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum DebugType<'a> {
    None,
    BaseSignal,
    Thermo,
    ForceFan,
    Sync,
    ModSegment,
    #[debug("ModIdx({})", _0)]
    ModIdx(u16),
    StmSegment,
    #[debug("StmIdx({})", _0)]
    StmIdx(u16),
    IsStmMode,
    #[debug("PwmOut({})", _0.local_idx())]
    PwmOut(&'a Transducer),
    #[debug("Direct({})", _0)]
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
            DebugType::PwmOut(tr) => tr.local_idx() as _,
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
    #[cfg_attr(miri, ignore)]
    fn display() {
        assert_eq!("None", format!("{:?}", DebugType::None));
        assert_eq!("BaseSignal", format!("{:?}", DebugType::BaseSignal));
        assert_eq!("Thermo", format!("{:?}", DebugType::Thermo));
        assert_eq!("ForceFan", format!("{:?}", DebugType::ForceFan));
        assert_eq!("Sync", format!("{:?}", DebugType::Sync));
        assert_eq!("ModSegment", format!("{:?}", DebugType::ModSegment));
        assert_eq!("ModIdx(1)", format!("{:?}", DebugType::ModIdx(1)));
        assert_eq!("StmSegment", format!("{:?}", DebugType::StmSegment));
        assert_eq!("StmIdx(1)", format!("{:?}", DebugType::StmIdx(1)));
        assert_eq!("IsStmMode", format!("{:?}", DebugType::IsStmMode));
        assert_eq!(
            "PwmOut(1)",
            format!(
                "{:?}",
                DebugType::PwmOut(&Transducer::new(1, 1, Vector3::new(0.0, 0.0, 0.0)))
            )
        );
        assert_eq!("Direct(true)", format!("{:?}", DebugType::Direct(true)));
    }
}
