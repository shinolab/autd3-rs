use crate::{ethercat::DcSysTime, geometry::Transducer};

use derive_more::Debug;

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
    /// High during the system time correction.
    SyncDiff,
    #[debug("PwmOut({})", _0.idx())]
    /// PWM output of the specified transducer.
    PwmOut(&'a Transducer),
    #[debug("Direct({})", _0)]
    /// High if `true`.
    Direct(bool),
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
