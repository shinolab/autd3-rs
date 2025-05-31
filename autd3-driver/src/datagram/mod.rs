mod boxed;
mod clear;
mod cpu_gpio_out;
mod force_fan;
mod fpga_gpio_out;
mod gain;
mod gpio_in;
mod group;
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
#[doc(hidden)]
pub mod v10;
mod with_loop_behavior;
mod with_segment;

#[doc(inline)]
pub use super::firmware::operation::SwapSegment;
#[doc(inline)]
pub use super::firmware::operation::{ControlPoint, ControlPoints};
pub use boxed::{BoxedDatagram, IntoBoxedDatagram};
pub use clear::Clear;
#[doc(hidden)]
pub use cpu_gpio_out::{CpuGPIOOutputs, CpuGPIOPort};
pub use force_fan::ForceFan;
pub use fpga_gpio_out::GPIOOutputs;
pub use gain::{BoxedGain, IntoBoxedGain};
#[doc(hidden)]
pub use gpio_in::EmulateGPIOIn;
pub use group::Group;
pub use modulation::{BoxedModulation, IntoBoxedModulation};
pub use phase_corr::PhaseCorrection;
pub use pulse_width_encoder::PulseWidthEncoder;
pub use reads_fpga_state::ReadsFPGAState;
pub use silencer::{FixedCompletionSteps, FixedCompletionTime, FixedUpdateRate, Silencer};
pub use stm::{
    FociSTM, FociSTMGenerator, FociSTMIterator, FociSTMIteratorGenerator, GainSTM,
    GainSTMGenerator, GainSTMIterator, GainSTMIteratorGenerator, GainSTMOption, STMConfig,
};
pub use with_loop_behavior::WithLoopBehavior;
pub use with_segment::WithSegment;

pub use synchronize::Synchronize;

pub use autd3_core::datagram::{Datagram, DeviceFilter};

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
