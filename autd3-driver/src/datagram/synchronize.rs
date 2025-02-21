use std::convert::Infallible;

use crate::firmware::operation::SyncOp;

use crate::datagram::*;

/// [`Datagram`] to synchronize the devices.
#[derive(Default, Debug)]
pub struct Synchronize {}

impl Synchronize {
    /// Creates a new [`Synchronize`] instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct SynchronizeOpGenerator {}

impl OperationGenerator for SynchronizeOpGenerator {
    type O1 = SyncOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(), Self::O2 {})
    }
}

impl Datagram for Synchronize {
    type G = SynchronizeOpGenerator;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: bool) -> Result<Self::G, Self::Error> {
        Ok(SynchronizeOpGenerator {})
    }
}
