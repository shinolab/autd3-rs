use std::convert::Infallible;

use crate::firmware::operation::ClearOp;

use crate::datagram::*;

/// [`Datagram`] to clear all data in the devices.
#[derive(Default, Debug)]
pub struct Clear {}

impl Clear {
    /// Creates a new [`Clear`].
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct ClearOpGenerator {}

impl OperationGenerator for ClearOpGenerator {
    type O1 = ClearOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new(), Self::O2 {}))
    }
}

impl Datagram for Clear {
    type G = ClearOpGenerator;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: &DeviceFilter) -> Result<Self::G, Self::Error> {
        Ok(ClearOpGenerator {})
    }
}
