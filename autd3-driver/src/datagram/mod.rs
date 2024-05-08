mod clear;
mod clk;
mod debug;
mod force_fan;
mod gain;
mod gpio_in;
mod modulation;
mod phase_filter;
mod pulse_width_encoder;
mod reads_fpga_state;
mod silencer;
mod stm;
mod synchronize;
mod with_segment;
mod with_segment_transition;
mod with_timeout;

pub use clear::Clear;
pub use clk::ConfigureFPGAClock;
pub use debug::ConfigureDebugSettings;
pub use force_fan::ConfigureForceFan;
pub use gain::{
    ChangeGainSegment, Gain, GainCache, GainFilter, GainTransform, Group, IntoGainCache,
    IntoGainTransform,
};
pub use gpio_in::EmulateGPIOIn;
pub use modulation::{
    ChangeModulationSegment, IntoModulationCache, IntoModulationTransform, IntoRadiationPressure,
    Modulation, ModulationCache, ModulationProperty, ModulationTransform, RadiationPressure,
};
pub use phase_filter::ConfigurePhaseFilter;
pub use pulse_width_encoder::ConfigurePulseWidthEncoder;
pub use reads_fpga_state::ConfigureReadsFPGAState;
pub use silencer::{
    ConfigureSilencer, ConfigureSilencerFixedCompletionSteps, ConfigureSilencerFixedUpdateRate,
};
pub use stm::{
    ChangeFocusSTMSegment, ChangeGainSTMSegment, ControlPoint, FocusSTM, GainSTM, STMProps,
};
pub use synchronize::Synchronize;
pub use with_segment::{DatagramS, DatagramWithSegment, IntoDatagramWithSegment};
pub use with_segment_transition::{
    DatagramST, DatagramWithSegmentTransition, IntoDatagramWithSegmentTransition,
};
pub use with_timeout::{DatagramWithTimeout, IntoDatagramWithTimeout};

use std::time::Duration;

use crate::{error::AUTDInternalError, firmware::operation::Operation};

/// Datagram to be sent to devices
pub trait Datagram {
    type O1: Operation;
    type O2: Operation;

    fn operation(self) -> (Self::O1, Self::O2);

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

impl<D1, D2> Datagram for (D1, D2)
where
    D1: Datagram<O2 = crate::firmware::operation::NullOp>,
    D2: Datagram<O2 = crate::firmware::operation::NullOp>,
{
    type O1 = D1::O1;
    type O2 = D2::O1;

    fn operation(self) -> (Self::O1, Self::O2) {
        let (o1, _) = self.0.operation();
        let (o2, _) = self.1.operation();
        (o1, o2)
    }

    fn timeout(&self) -> Option<Duration> {
        match (self.0.timeout(), self.1.timeout()) {
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
            (Some(t1), None) => Some(t1),
            (None, Some(t2)) => Some(t2),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::operation::{ClearOp, NullOp};

    use super::*;

    struct TestDatagram1 {}
    impl Datagram for TestDatagram1 {
        type O1 = ClearOp;
        type O2 = NullOp;

        fn operation(self) -> (Self::O1, Self::O2) {
            (Self::O1::default(), Self::O2::default())
        }
    }

    struct TestDatagram2 {}
    impl Datagram for TestDatagram2 {
        type O1 = NullOp;
        type O2 = NullOp;

        fn operation(self) -> (Self::O1, Self::O2) {
            (Self::O1::default(), Self::O2::default())
        }
    }

    #[test]
    fn test_tuple() {
        let d = (TestDatagram1 {}, TestDatagram2 {});
        assert_eq!(None, d.timeout());
        let _: (ClearOp, NullOp) = <(TestDatagram1, TestDatagram2) as Datagram>::operation(d);
    }

    struct TestDatagramWithTimeout {
        pub timeout: Option<Duration>,
    }
    impl Datagram for TestDatagramWithTimeout {
        type O1 = ClearOp;
        type O2 = NullOp;

        // GRCOV_EXCL_START
        fn operation(self) -> (Self::O1, Self::O2) {
            unimplemented!()
        }
        // GRCOV_EXCL_STOP

        fn timeout(&self) -> Option<Duration> {
            self.timeout
        }
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Some(Duration::from_secs(2)),
        Some(Duration::from_secs(2)),
        Some(Duration::from_secs(1))
    )]
    #[case(Some(Duration::from_secs(1)), Some(Duration::from_secs(1)), None)]
    #[case(Some(Duration::from_secs(1)), None, Some(Duration::from_secs(1)))]
    #[case(None, None, None)]
    fn test_tuple_timeout(
        #[case] expext: Option<Duration>,
        #[case] t1: Option<Duration>,
        #[case] t2: Option<Duration>,
    ) {
        assert_eq!(
            expext,
            (
                TestDatagramWithTimeout { timeout: t1 },
                TestDatagramWithTimeout { timeout: t2 },
            )
                .timeout()
        );
    }
}
