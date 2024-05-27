use crate::{datagram::*, derive::DEFAULT_TIMEOUT};

#[derive(Default)]
pub struct ConfigureFPGAClock {}

impl ConfigureFPGAClock {
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct ConfigureClockOpGenerator {}

impl<'a> OperationGenerator<'a> for ConfigureClockOpGenerator {
    type O1 = crate::firmware::operation::ConfigureClockOp;
    type O2 = crate::firmware::operation::NullOp;

    fn generate(&'a self, _: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::default(), Self::O2::default()))
    }
}

impl<'a> Datagram<'a> for ConfigureFPGAClock {
    type O1 = crate::firmware::operation::ConfigureClockOp;
    type O2 = crate::firmware::operation::NullOp;
    type G =  ConfigureClockOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &'a Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ConfigureClockOpGenerator {})
    }
}
