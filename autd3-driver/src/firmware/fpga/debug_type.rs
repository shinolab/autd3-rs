use crate::{defined::ULTRASOUND_PERIOD, ethercat::DcSysTime, geometry::Transducer};

use derive_more::Debug;
use zerocopy::{Immutable, IntoBytes};

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
    SysTimeEq(DcSysTime),
    #[debug("PwmOut({})", _0.idx())]
    PwmOut(&'a Transducer),
    #[debug("Direct({})", _0)]
    Direct(bool),
}

#[bitfield_struct::bitfield(u64)]
#[derive(IntoBytes, Immutable)]
pub(crate) struct DebugValue {
    #[bits(56)]
    pub(crate) value: u64,
    #[bits(8)]
    pub(crate) tag: u8,
}

impl From<DebugType<'_>> for DebugValue {
    fn from(ty: DebugType<'_>) -> Self {
        Self::new()
            .with_value(match &ty {
                DebugType::None
                | DebugType::BaseSignal
                | DebugType::Thermo
                | DebugType::ForceFan
                | DebugType::Sync
                | DebugType::ModSegment
                | DebugType::StmSegment
                | DebugType::IsStmMode => 0,
                DebugType::PwmOut(tr) => tr.idx() as _,
                DebugType::ModIdx(idx) => *idx as _,
                DebugType::StmIdx(idx) => *idx as _,
                DebugType::SysTimeEq(time) => {
                    (time.sys_time() / ULTRASOUND_PERIOD.as_nanos() as u64) << 8
                }
                DebugType::Direct(v) => *v as _,
            })
            .with_tag(match &ty {
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
                DebugType::SysTimeEq(_) => 0x60,
                DebugType::PwmOut(_) => 0xE0,
                DebugType::Direct(_) => 0xF0,
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::Vector3;

    use super::*;

    #[test]
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
