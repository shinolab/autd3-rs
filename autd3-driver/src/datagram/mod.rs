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

#[doc(inline)]
pub use super::firmware::operation::SwapSegment;
#[doc(inline)]
pub use super::firmware::operation::{ControlPoint, ControlPoints};
pub use clear::Clear;
#[doc(hidden)]
pub use cpu_gpio_out::{CpuGPIO, CpuGPIOPort};
pub use debug::DebugSettings;
pub use force_fan::ForceFan;
pub use gain::{BoxedGain, Gain, GainContextGenerator, GainOperationGenerator, IntoBoxedGain};
#[doc(hidden)]
pub use gpio_in::EmulateGPIOIn;
pub use modulation::{
    BoxedModulation, IntoBoxedModulation, Modulation, ModulationOperationGenerator,
    ModulationProperty,
};
pub use phase_corr::PhaseCorrection;
pub use pulse_width_encoder::PulseWidthEncoder;
pub use reads_fpga_state::ReadsFPGAState;
pub use silencer::{FixedCompletionTime, FixedUpdateRate, HasSamplingConfig, Silencer};
pub use stm::{
    FociSTM, FociSTMContext, FociSTMContextGenerator, FociSTMGenerator, GainSTM, GainSTMContext,
    GainSTMContextGenerator, GainSTMGenerator, IntoFociSTMGenerator, IntoGainSTMGenerator,
    STMConfig, STMConfigNearest,
};
pub use synchronize::Synchronize;
pub use with_parallel_threshold::{
    DatagramWithParallelThreshold, IntoDatagramWithParallelThreshold,
};
pub use with_segment::{DatagramS, DatagramWithSegment, IntoDatagramWithSegment};
pub use with_timeout::{DatagramWithTimeout, IntoDatagramWithTimeout};

use crate::{
    defined::DEFAULT_TIMEOUT,
    firmware::operation::NullOp,
    geometry::{Device, Geometry},
};
use std::time::Duration;

use crate::{error::AUTDDriverError, firmware::operation::OperationGenerator};

/// [`Datagram`] represents the data sent to the device.
pub trait Datagram: std::fmt::Debug {
    #[doc(hidden)]
    type G: OperationGenerator;

    #[doc(hidden)]
    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDDriverError>;

    /// Returns the timeout duration.
    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    /// Returns the parallel threshold.
    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::firmware::fpga::{Segment, TransitionMode};

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
            (Self::O1::new(), Self::O2::new())
        }
        // GRCOV_EXCL_STOP
    }

    impl DatagramS for NullDatagram {
        type G = NullOperationGenerator;

        fn operation_generator_with_segment(
            self,
            _: &Geometry,
            _segment: Segment,
            _transition_mode: Option<TransitionMode>,
        ) -> Result<Self::G, AUTDDriverError> {
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
