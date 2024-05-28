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

impl<'a> OperationGenerator for ConfigureClockOpGenerator {
    type O1 = ConfigureClockOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::default(), Self::O2::default()))
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
}
