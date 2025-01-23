use std::convert::Infallible;

use crate::firmware::operation::ClearOp;

use crate::datagram::*;
use derive_new::new;

/// [`Datagram`] to clear all data in the devices.
#[derive(Default, Debug, new)]
pub struct Clear {}

pub struct ClearOpGenerator {}

impl OperationGenerator for ClearOpGenerator {
    type O1 = ClearOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(), Self::O2 {})
    }
}

impl Datagram for Clear {
    type G = ClearOpGenerator;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: bool) -> Result<Self::G, Self::Error> {
        Ok(ClearOpGenerator {})
    }
}
