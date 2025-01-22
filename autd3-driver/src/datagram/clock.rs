use std::convert::Infallible;

use crate::{datagram::*, defined::ultrasound_freq, firmware::operation::ConfigureClockOp};
use autd3_core::derive::DatagramOption;

#[derive(Default, Debug)]
#[doc(hidden)]
pub struct ConfigureFPGAClock {}

impl ConfigureFPGAClock {
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct ConfigureClockOpGenerator {}

impl OperationGenerator for ConfigureClockOpGenerator {
    type O1 = ConfigureClockOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(ultrasound_freq()), Self::O2 {})
    }
}

impl Datagram for ConfigureFPGAClock {
    type G = ConfigureClockOpGenerator;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: &DatagramOption) -> Result<Self::G, Self::Error> {
        Ok(ConfigureClockOpGenerator {})
    }
}
