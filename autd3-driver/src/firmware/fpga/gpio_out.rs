use crate::{ethercat::DcSysTime, geometry::Transducer};

use derive_more::Debug;
use zerocopy::{Immutable, IntoBytes};

use super::ec_time_to_sys_time;

/// Output of the GPIO pin. See also [`GPIOOutputs`].
///
/// [`GPIOOutputs`]: crate::datagram::GPIOOutputs
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum GPIOOutputType<'a> {
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
    /// High during the specified system time or later.
    SysTimeGe(DcSysTime),
    /// High during the system time correction.
    SyncDiff,
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

impl From<Option<GPIOOutputType<'_>>> for DebugValue {
    fn from(ty: Option<GPIOOutputType<'_>>) -> Self {
        Self::new()
            .with_value(match &ty {
                None
                | Some(GPIOOutputType::BaseSignal)
                | Some(GPIOOutputType::Thermo)
                | Some(GPIOOutputType::ForceFan)
                | Some(GPIOOutputType::Sync)
                | Some(GPIOOutputType::ModSegment)
                | Some(GPIOOutputType::StmSegment)
                | Some(GPIOOutputType::IsStmMode)
                | Some(GPIOOutputType::SyncDiff) => 0,
                Some(GPIOOutputType::PwmOut(tr)) => tr.idx() as _,
                Some(GPIOOutputType::ModIdx(idx)) | Some(GPIOOutputType::StmIdx(idx)) => *idx as _,
                Some(GPIOOutputType::SysTimeEq(time)) | Some(GPIOOutputType::SysTimeGe(time)) => {
                    ec_time_to_sys_time(time) >> 9
                }
                Some(GPIOOutputType::Direct(v)) => *v as _,
            })
            .with_tag(match &ty {
                None => 0x00,
                Some(GPIOOutputType::BaseSignal) => 0x01,
                Some(GPIOOutputType::Thermo) => 0x02,
                Some(GPIOOutputType::ForceFan) => 0x03,
                Some(GPIOOutputType::Sync) => 0x10,
                Some(GPIOOutputType::ModSegment) => 0x20,
                Some(GPIOOutputType::ModIdx(_)) => 0x21,
                Some(GPIOOutputType::StmSegment) => 0x50,
                Some(GPIOOutputType::StmIdx(_)) => 0x51,
                Some(GPIOOutputType::IsStmMode) => 0x52,
                Some(GPIOOutputType::SysTimeEq(_)) => 0x60,
                Some(GPIOOutputType::SysTimeGe(_)) => 0x61,
                Some(GPIOOutputType::SyncDiff) => 0x70,
                Some(GPIOOutputType::PwmOut(_)) => 0xE0,
                Some(GPIOOutputType::Direct(_)) => 0xF0,
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::Point3;

    use super::*;

    #[test]
    fn display() {
        assert_eq!("BaseSignal", format!("{:?}", GPIOOutputType::BaseSignal));
        assert_eq!("Thermo", format!("{:?}", GPIOOutputType::Thermo));
        assert_eq!("ForceFan", format!("{:?}", GPIOOutputType::ForceFan));
        assert_eq!("Sync", format!("{:?}", GPIOOutputType::Sync));
        assert_eq!("ModSegment", format!("{:?}", GPIOOutputType::ModSegment));
        assert_eq!("ModIdx(1)", format!("{:?}", GPIOOutputType::ModIdx(1)));
        assert_eq!("StmSegment", format!("{:?}", GPIOOutputType::StmSegment));
        assert_eq!("StmIdx(1)", format!("{:?}", GPIOOutputType::StmIdx(1)));
        assert_eq!("IsStmMode", format!("{:?}", GPIOOutputType::IsStmMode));
        assert_eq!(
            "PwmOut(0)",
            format!(
                "{:?}",
                GPIOOutputType::PwmOut(&Transducer::new(Point3::origin()))
            )
        );
        assert_eq!(
            "Direct(true)",
            format!("{:?}", GPIOOutputType::Direct(true))
        );
    }
}
