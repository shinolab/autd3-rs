mod clear;
#[cfg(feature = "dynamic_freq")]
mod clock;
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
#[cfg(feature = "dynamic_freq")]
pub use clock::ConfigureFPGAClock;
#[doc(hidden)]
pub use cpu_gpio_out::{CpuGPIO, CpuGPIOPort};
pub use debug::DebugSettings;
pub use force_fan::ForceFan;
pub use gain::{BoxedGain, IntoBoxedGain};
#[doc(hidden)]
pub use gpio_in::EmulateGPIOIn;
pub use modulation::{BoxedModulation, IntoBoxedModulation};
pub use phase_corr::PhaseCorrection;
pub use pulse_width_encoder::PulseWidthEncoder;
pub use reads_fpga_state::ReadsFPGAState;
#[cfg(not(feature = "dynamic_freq"))]
pub use silencer::FixedCompletionTime;
pub use silencer::{FixedCompletionSteps, FixedUpdateRate, HasSamplingConfig, Silencer};
pub use stm::{
    FociSTM, FociSTMContext, FociSTMContextGenerator, FociSTMGenerator, GainSTM, GainSTMContext,
    GainSTMContextGenerator, GainSTMGenerator, IntoFociSTMGenerator, IntoGainSTMGenerator,
    STMConfig, STMConfigNearest,
};
pub use synchronize::Synchronize;
pub use with_parallel_threshold::{
    DatagramWithParallelThreshold, IntoDatagramWithParallelThreshold,
};
pub use with_segment::{DatagramWithSegment, IntoDatagramWithSegment};
pub use with_timeout::{DatagramWithTimeout, IntoDatagramWithTimeout};

pub use autd3_core::datagram::Datagram;

use crate::{
    firmware::operation::NullOp,
    geometry::{Device, Geometry},
};

use crate::{error::AUTDDriverError, firmware::operation::OperationGenerator};

#[cfg(test)]
pub(crate) mod tests {
    use std::time::Duration;

    use autd3_core::datagram::DatagramS;

    use crate::firmware::{
        fpga::{Segment, TransitionMode},
        operation::tests::create_device,
    };

    use super::*;

    pub fn create_geometry(n: u16, num_trans_in_unit: u8) -> Geometry {
        Geometry::new(
            (0..n)
                .map(|i| create_device(i, num_trans_in_unit))
                .collect(),
            4,
        )
    }

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
            (Self::O1 {}, Self::O2 {})
        }
        // GRCOV_EXCL_STOP
    }

    impl DatagramS for NullDatagram {
        type G = NullOperationGenerator;
        type Error = AUTDDriverError;

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
