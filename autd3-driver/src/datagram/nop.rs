use std::convert::Infallible;

use crate::{datagram::*, firmware::operation::NopOp};

use derive_more::Debug;

/// [`Datagram`] which does nothing.
#[derive(Debug)]
pub struct Nop;

impl OperationGenerator for Nop {
    type O1 = NopOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(), Self::O2 {}))
    }
}

impl Datagram for Nop {
    type G = Nop;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: &DeviceFilter) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
