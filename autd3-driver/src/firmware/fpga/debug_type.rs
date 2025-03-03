use crate::{ethercat::DcSysTime, geometry::Transducer};

use derive_more::Debug;
use zerocopy::{Immutable, IntoBytes};

use super::ec_time_to_sys_time;

/// Output of the GPIO pin. See also [`DebugSettings`].
///
/// [`DebugSettings`]: crate::datagram::DebugSettings
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum DebugType<'a> {
    /// Do not output.
    None,
    /// Base signal (50% duty cycle square wave with the same frequency as ultrasound).
    BaseSignal,
    /// High if the temperature sensor is asserted.
    Thermo,
    /// High if the ForceFan flag is asserted.
    ForceFan,
    /// EtherCAT synchronization signal.
    Sync,
    /// Modulation segment (High if the segment is 1, Low if the segment is 0).
    ModSegment,
    #[debug("ModIdx({})", _0)]
    /// High when the Modulation index is the specified value.
    ModIdx(u16),
    /// STM and Gain segment (High if the segment is 1, Low if the segment is 0).
    StmSegment,
    #[debug("StmIdx({})", _0)]
    /// High when the STM index is the specified value.
    StmIdx(u16),
    /// High if FociSTM/GainSTM is used.
    IsStmMode,
    /// High during the specified system time.
    SysTimeEq(DcSysTime),
    #[debug("PwmOut({})", _0.idx())]
    /// PWM output of the specified transducer.
    PwmOut(&'a Transducer),
    #[debug("Direct({})", _0)]
    /// High if `true`.
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
                DebugType::SysTimeEq(time) => ec_time_to_sys_time(time),
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
    use crate::geometry::Point3;

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
            "PwmOut(0)",
            format!(
                "{:?}",
                DebugType::PwmOut(&Transducer::new(Point3::origin()))
            )
        );
        assert_eq!("Direct(true)", format!("{:?}", DebugType::Direct(true)));
    }
}
