mod clear;
mod cpu_gpio_out;
mod debug;
mod force_fan;
mod gain;
mod gpio_in;
mod info;
mod modulation;
mod phase_corr;
mod pulse_width_encoder;
mod reads_fpga_state;
mod segment;
mod silencer;
mod stm;
mod synchronize;
mod tuple;
mod with_parallel_threshold;
mod with_segment;
mod with_timeout;

pub use clear::Clear;
pub use cpu_gpio_out::{CpuGPIO, CpuGPIOPort};
pub use debug::DebugSettings;
pub use force_fan::ForceFan;
pub use gain::{BoxedGain, Gain, GainContextGenerator, GainOperationGenerator, IntoBoxedGain};
pub use gpio_in::EmulateGPIOIn;
pub use modulation::{
    BoxedModulation, IntoBoxedModulation, Modulation, ModulationOperationGenerator,
    ModulationProperty,
};
pub use phase_corr::PhaseCorrection;
pub use pulse_width_encoder::PulseWidthEncoder;
pub use reads_fpga_state::ReadsFPGAState;
pub use segment::SwapSegment;
pub use silencer::{FixedCompletionTime, FixedUpdateRate, Silencer, WithSampling};
pub use stm::{FociSTM, GainSTM, STMConfig, STMConfigNearest};
pub use synchronize::Synchronize;
pub use with_parallel_threshold::{
    DatagramWithParallelThreshold, IntoDatagramWithParallelThreshold,
};
pub use with_segment::{DatagramS, DatagramWithSegment, IntoDatagramWithSegment};
pub use with_timeout::{DatagramWithTimeout, IntoDatagramWithTimeout};

use crate::{defined::DEFAULT_TIMEOUT, firmware::operation::NullOp, geometry::Device};
use std::time::Duration;

use crate::{derive::Geometry, error::AUTDInternalError, firmware::operation::OperationGenerator};

pub trait Datagram: std::fmt::Debug {
    type G: OperationGenerator;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError>;
    fn timeout(&self) -> Option<Duration>;
    fn parallel_threshold(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
pub mod tests {
    use crate::derive::{Segment, TransitionMode};

    use super::*;

    #[derive(Debug)]
    pub struct NullDatagram {
        pub timeout: Option<Duration>,
        pub parallel_threshold: Option<usize>,
    }

    pub struct NullOperationGenerator {}

    impl OperationGenerator for NullOperationGenerator {
        type O1 = crate::firmware::operation::NullOp;
        type O2 = crate::firmware::operation::NullOp;

        // GRCOV_EXCL_START
        fn generate(&mut self, _device: &Device) -> (Self::O1, Self::O2) {
            (Self::O1::default(), Self::O2::default())
        }
        // GRCOV_EXCL_STOP
    }

    impl DatagramS for NullDatagram {
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
