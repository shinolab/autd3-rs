use crate::{defined::ultrasound_freq, firmware::operation::ConfigureClockOp};

use crate::datagram::*;

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
        (Self::O1::new(ultrasound_freq()), Self::O2::new())
    }
}

impl Datagram for ConfigureFPGAClock {
    type G = ConfigureClockOpGenerator;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDDriverError> {
        Ok(ConfigureClockOpGenerator {})
    }
}
