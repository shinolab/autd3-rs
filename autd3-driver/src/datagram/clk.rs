use crate::{datagram::*, derive::DEFAULT_TIMEOUT};

#[derive(Default)]
pub struct ConfigureFPGAClock {}

impl ConfigureFPGAClock {
    pub const fn new() -> Self {
        Self {}
    }
}

impl<'a> Datagram<'a> for ConfigureFPGAClock {
    type O1 = crate::firmware::operation::ConfigureClockOp;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(
        &'a self,
        _: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2), AUTDInternalError> {
        Ok(move |_| (Self::O1::default(), Self::O2::default()))
    }
}
