use crate::{datagram::*, defined::DEFAULT_TIMEOUT};

#[derive(Default)]
pub struct Synchronize {}

impl Synchronize {
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct SynchronizeOpGenerator {}

impl<'a> OperationGenerator<'a> for SynchronizeOpGenerator {
    type O1 = crate::firmware::operation::SyncOp;
    type O2 = crate::firmware::operation::NullOp;

    fn generate(&'a self, _: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::default(), Self::O2::default()))
    }
}

impl<'a> Datagram<'a> for Synchronize {
    type O1 = crate::firmware::operation::SyncOp;
    type O2 = crate::firmware::operation::NullOp;
    type G = SynchronizeOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &'a Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SynchronizeOpGenerator {})
    }
}
