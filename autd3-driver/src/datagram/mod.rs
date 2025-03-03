mod boxed;
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
mod with_loop_behavior;
mod with_segment;

#[doc(inline)]
pub use super::firmware::operation::SwapSegment;
#[doc(inline)]
pub use super::firmware::operation::{ControlPoint, ControlPoints};
pub use boxed::{BoxedDatagram, IntoBoxedDatagram};
pub use clear::Clear;
#[cfg(feature = "dynamic_freq")]
pub use clock::ConfigureFPGAClock;
#[doc(hidden)]
pub use cpu_gpio_out::{CpuGPIOOutputs, CpuGPIOPort};
pub use debug::GPIOOutputs;
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
pub use silencer::{FixedCompletionSteps, FixedUpdateRate, Silencer};
pub use stm::{
    FociSTM, FociSTMGenerator, FociSTMIterator, FociSTMIteratorGenerator, GainSTM,
    GainSTMGenerator, GainSTMIterator, GainSTMIteratorGenerator, GainSTMOption, STMConfig,
};
pub use with_loop_behavior::WithLoopBehavior;
pub use with_segment::WithSegment;

pub use synchronize::Synchronize;

pub use autd3_core::datagram::Datagram;

use crate::{
    firmware::operation::NullOp,
    geometry::{Device, Geometry},
};

use crate::{error::AUTDDriverError, firmware::operation::OperationGenerator};

#[cfg(test)]
pub(crate) mod tests {
    use crate::firmware::operation::tests::create_device;

    use super::*;

    pub fn create_geometry(n: u16, num_trans_in_unit: u8) -> Geometry {
        Geometry::new((0..n).map(|_| create_device(num_trans_in_unit)).collect())
    }
}
