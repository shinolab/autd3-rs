use crate::firmware::operation::SyncOp;

use crate::datagram::*;

#[derive(Default)]
pub struct Synchronize {}

impl Synchronize {
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct SynchronizeOpGenerator {}

impl OperationGenerator for SynchronizeOpGenerator {
    type O1 = SyncOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::default(), Self::O2::default()))
    }
}

impl<'a> Datagram<'a> for Synchronize {
    type O1 = SyncOp;
    type O2 = NullOp;
    type G = SynchronizeOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SynchronizeOpGenerator {})
    }
}
