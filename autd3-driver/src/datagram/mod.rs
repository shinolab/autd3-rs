mod clear;
mod debug;
mod force_fan;
mod gain;
mod modulation;
mod reads_fpga_state;
mod silencer;
mod stm;
mod synchronize;
mod with_segment;
mod with_timeout;

pub use clear::Clear;
pub use debug::ConfigureDebugOutputIdx;
pub use force_fan::ConfigureForceFan;
pub use gain::{
    Gain, GainCache, GainFilter, GainTransform, Group, IntoGainCache, IntoGainTransform,
};
pub use modulation::{
    IntoModulationCache, IntoModulationTransform, IntoRadiationPressure, Modulation,
    ModulationCache, ModulationProperty, ModulationTransform, RadiationPressure,
};
pub use reads_fpga_state::ConfigureReadsFPGAState;
pub use silencer::{
    ConfigureSilencer, ConfigureSilencerFixedCompletionSteps, ConfigureSilencerFixedUpdateRate,
};
pub use stm::{FocusSTM, GainSTM, STMProps};
pub use synchronize::Synchronize;
pub use with_segment::{DatagramS, DatagramWithSegment, IntoDatagramWithSegment};
pub use with_timeout::{DatagramWithTimeout, IntoDatagramWithTimeout};

use std::time::Duration;

use crate::{error::AUTDInternalError, operation::Operation};

/// Datagram to be sent to devices
pub trait Datagram {
    type O1: Operation;
    type O2: Operation;

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

impl<D1, D2> Datagram for (D1, D2)
where
    D1: Datagram<O2 = crate::operation::NullOp>,
    D2: Datagram<O2 = crate::operation::NullOp>,
{
    type O1 = D1::O1;
    type O2 = D2::O1;

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        let (o1, _) = self.0.operation()?;
        let (o2, _) = self.1.operation()?;
        Ok((o1, o2))
    }
}

#[cfg(test)]
mod tests {
    use crate::operation::{ClearOp, NullOp};

    use super::*;

    struct TestDatagram1 {
        pub err: bool,
    }
    impl Datagram for TestDatagram1 {
        type O1 = ClearOp;
        type O2 = NullOp;

        fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
            if self.err {
                Err(AUTDInternalError::NotSupported("Err1".to_owned()))
            } else {
                Ok((Self::O1::default(), Self::O2::default()))
            }
        }
    }

    struct TestDatagram2 {
        pub err: bool,
    }
    impl Datagram for TestDatagram2 {
        type O1 = NullOp;
        type O2 = NullOp;

        fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
            if self.err {
                Err(AUTDInternalError::NotSupported("Err2".to_owned()))
            } else {
                Ok((Self::O1::default(), Self::O2::default()))
            }
        }
    }

    #[test]
    fn test_datagram_tuple() {
        let d = (TestDatagram1 { err: false }, TestDatagram2 { err: false });
        let _: (ClearOp, NullOp) =
            <(TestDatagram1, TestDatagram2) as Datagram>::operation(d).unwrap();
    }

    #[test]
    fn test_datagram_tuple_err() {
        let d1 = (TestDatagram1 { err: true }, TestDatagram2 { err: false });
        let r = <(TestDatagram1, TestDatagram2) as Datagram>::operation(d1);
        assert!(r.is_err());

        let d2 = (TestDatagram1 { err: false }, TestDatagram2 { err: true });
        let r = <(TestDatagram1, TestDatagram2) as Datagram>::operation(d2);
        assert!(r.is_err());
    }
}
