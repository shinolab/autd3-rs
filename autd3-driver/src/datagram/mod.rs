mod clear;
#[cfg(feature = "dynamic_freq")]
mod clk;
mod debug;
mod force_fan;
mod gain;
mod gpio_in;
mod modulation;
mod pulse_width_encoder;
mod reads_fpga_state;
pub mod segment;
mod silencer;
mod stm;
mod synchronize;
mod with_parallel_threshold;
mod with_segment;
mod with_segment_transition;
mod with_timeout;

pub use clear::Clear;
#[cfg(feature = "dynamic_freq")]
pub use clk::ConfigureFPGAClock;
pub use debug::DebugSettings;
pub use force_fan::ForceFan;
pub use gain::{
    Gain, GainCache, GainCalcResult, GainOperationGenerator, GainTransform, Group, IntoGainCache,
    IntoGainTransform,
};
pub use gpio_in::EmulateGPIOIn;
pub use modulation::{
    IntoModulationCache, IntoModulationTransform, IntoRadiationPressure, Modulation,
    ModulationCache, ModulationCalcResult, ModulationOperationGenerator, ModulationProperty,
    ModulationTransform, RadiationPressure,
};
pub use pulse_width_encoder::PulseWidthEncoder;
pub use reads_fpga_state::ReadsFPGAState;
pub use segment::SwapSegment;
pub use silencer::{
    Silencer, SilencerFixedCompletionSteps, SilencerFixedCompletionTime, SilencerFixedUpdateRate,
};
pub use stm::{FociSTM, GainSTM};
pub use synchronize::Synchronize;
pub use with_parallel_threshold::{
    DatagramWithParallelThreshold, IntoDatagramWithParallelThreshold,
};
pub use with_segment::{DatagramS, DatagramWithSegment, IntoDatagramWithSegment};
pub use with_segment_transition::{
    DatagramST, DatagramWithSegmentTransition, IntoDatagramWithSegmentTransition,
};
pub use with_timeout::{DatagramWithTimeout, IntoDatagramWithTimeout};

use crate::{defined::DEFAULT_TIMEOUT, firmware::operation::NullOp};
use std::time::Duration;

use crate::{
    derive::{Device, Geometry},
    error::AUTDInternalError,
    firmware::operation::OperationGenerator,
};

pub trait Datagram {
    type G: OperationGenerator;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }

    fn parallel_threshold(&self) -> Option<usize> {
        None
    }

    #[tracing::instrument(skip(self, _geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("Datagram");
    }
    // GRCOV_EXCL_STOP
}

pub struct CombinedOperationGenerator<O1, O2>
where
    O1: OperationGenerator,
    O2: OperationGenerator,
{
    o1: O1,
    o2: O2,
}

impl<O1, O2> OperationGenerator for CombinedOperationGenerator<O1, O2>
where
    O1: OperationGenerator,
    O2: OperationGenerator,
{
    type O1 = O1::O1;
    type O2 = O2::O1;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        let (o1, _) = self.o1.generate(device);
        let (o2, _) = self.o2.generate(device);
        (o1, o2)
    }
}

impl<G1, G2, D1, D2> Datagram for (D1, D2)
where
    D1: Datagram<G = G1>,
    D2: Datagram<G = G2>,
    G1: OperationGenerator<O2 = NullOp>,
    G2: OperationGenerator<O2 = NullOp>,
{
    type G = CombinedOperationGenerator<D1::G, D2::G>;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(CombinedOperationGenerator {
            o1: self.0.operation_generator(geometry)?,
            o2: self.1.operation_generator(geometry)?,
        })
    }

    fn timeout(&self) -> Option<Duration> {
        match (self.0.timeout(), self.1.timeout()) {
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
            (a, b) => a.or(b),
        }
    }

    fn parallel_threshold(&self) -> Option<usize> {
        match (self.0.parallel_threshold(), self.1.parallel_threshold()) {
            (Some(t1), Some(t2)) => Some(t1.min(t2)),
            (a, b) => a.or(b),
        }
    }

    #[tracing::instrument(skip(self, geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("Datagram (tuple)");
        self.0.trace(geometry);
        self.1.trace(geometry);
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
pub mod tests {
    use crate::derive::{Segment, TransitionMode};

    use super::*;

    pub struct NullDatagram {
        pub timeout: Option<Duration>,
        pub parallel_threshold: Option<usize>,
    }

    pub struct NullOperationGenerator {}

    impl OperationGenerator for NullOperationGenerator {
        type O1 = crate::firmware::operation::NullOp;
        type O2 = crate::firmware::operation::NullOp;

        // GRCOV_EXCL_START
        fn generate(&self, _device: &crate::derive::Device) -> (Self::O1, Self::O2) {
            (Self::O1::default(), Self::O2::default())
        }
        // GRCOV_EXCL_STOP
    }

    impl DatagramST for NullDatagram {
        type G = NullOperationGenerator;

        fn operation_generator_with_segment(
            self,
            _: &crate::derive::Geometry,
            _segment: Segment,
            _transition_mode: Option<TransitionMode>,
        ) -> Result<Self::G, crate::derive::AUTDInternalError> {
            Ok(NullOperationGenerator {})
        }

        fn timeout(&self) -> Option<Duration> {
            self.timeout
        }

        fn parallel_threshold(&self) -> Option<usize> {
            self.parallel_threshold
        }
    }
}
