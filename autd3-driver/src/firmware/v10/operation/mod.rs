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
mod sync;
mod tuple;

pub(crate) use clear::*;
pub(crate) use cpu_gpio_out::*;
pub(crate) use force_fan::*;
pub(crate) use fpga_gpio_out::*;
pub(crate) use gain::*;
pub(crate) use gpio_in::*;
pub(crate) use info::*;
pub(crate) use modulation::*;
pub(crate) use phase_corr::*;
pub(crate) use pulse_width_encoder::*;
pub(crate) use reads_fpga_state::*;
pub(crate) use segment::*;
pub(crate) use silencer::*;
pub(crate) use stm::*;
pub(crate) use sync::*;

use crate::firmware::driver::Operation;

use autd3_core::geometry::Device;

#[doc(hidden)]
pub trait OperationGenerator<'a> {
    type O1: Operation<'a>;
    type O2: Operation<'a>;

    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)>;
}
