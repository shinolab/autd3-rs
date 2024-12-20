use crate::firmware::operation::ClearOp;

use crate::datagram::*;
use derive_new::new;

#[derive(Default, Debug, new)]
pub struct Clear {}

pub struct ClearOpGenerator {}

impl OperationGenerator for ClearOpGenerator {
    type O1 = ClearOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(), Self::O2::new())
    }
}

impl Datagram for Clear {
    type G = ClearOpGenerator;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDDriverError> {
        Ok(ClearOpGenerator {})
    }
}
