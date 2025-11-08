use crate::{ethercat::DcSysTime, geometry::Transducer};

/// Output of the GPIO pin. See also [`GPIOOutputs`].
///
/// [`GPIOOutputs`]: crate::datagram::GPIOOutputs
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum GPIOOutputType<'a> {
    /// Base signal (50% duty ratio square wave with the frequency of the ultrasound).
    BaseSignal,
    /// High if the temperature sensor is asserted.
    Thermo,
    /// High if the ForceFan flag is asserted.
    ForceFan,
    /// EtherCAT synchronization signal.
    Sync,
    /// Modulation segment (High if the segment is 1, Low if the segment is 0).
    ModSegment,
    /// High during the specified modulation index.
    ModIdx(u16),
    /// STM and Gain segment (High if the segment is 1, Low if the segment is 0).
    StmSegment,
    /// High during the specified STM index.
    StmIdx(u16),
    /// High if FociSTM/GainSTM is used.
    IsStmMode,
    /// High during the specified system time.
    SysTimeEq(DcSysTime),
    /// High during the system time correction.
    SyncDiff,
    /// PWM output of the specified transducer.
    PwmOut(&'a Transducer),
    /// High if `true`.
    Direct(bool),
}
