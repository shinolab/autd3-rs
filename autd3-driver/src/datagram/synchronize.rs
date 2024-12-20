use crate::firmware::operation::SyncOp;

use crate::datagram::*;
use derive_new::new;

#[derive(Default, Debug, new)]
pub struct Synchronize {}

pub struct SynchronizeOpGenerator {}

impl OperationGenerator for SynchronizeOpGenerator {
    type O1 = SyncOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(), Self::O2::new())
    }
}

impl Datagram for Synchronize {
    type G = SynchronizeOpGenerator;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDDriverError> {
        Ok(SynchronizeOpGenerator {})
    }
}
