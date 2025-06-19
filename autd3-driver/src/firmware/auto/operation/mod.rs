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
mod nop;
mod phase_corr;
mod pulse_width_encoder;
mod reads_fpga_state;
mod segment;
mod silencer;
mod stm;
mod sync;
mod tuple;

use crate::{
    firmware::driver::{Operation, Version},
    geometry::Device,
};

#[doc(hidden)]
pub trait OperationGenerator {
    type O1: Operation;
    type O2: Operation;

    #[must_use]
    fn generate(&mut self, device: &Device, version: Version) -> Option<(Self::O1, Self::O2)>;
}
