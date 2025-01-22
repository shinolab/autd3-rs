use std::convert::Infallible;

use crate::firmware::operation::SyncOp;

use crate::datagram::*;
use autd3_core::datagram::DatagramOption;
use derive_new::new;

/// [`Datagram`] to synchronize the devices.
#[derive(Default, Debug, new)]
pub struct Synchronize {}

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

    fn operation_generator(self, _: &Geometry, _: &DatagramOption) -> Result<Self::G, Self::Error> {
        Ok(SynchronizeOpGenerator {})
    }
}
