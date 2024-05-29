use crate::firmware::operation::ConfigureClockOp;

use crate::datagram::*;

#[derive(Default)]
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

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(device.ultrasound_freq()), Self::O2::default())
    }
}

impl<'a> Datagram<'a> for ConfigureFPGAClock {
    type O1 = ConfigureClockOp;
    type O2 = NullOp;
    type G = ConfigureClockOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ConfigureClockOpGenerator {})
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}
