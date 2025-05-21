use std::convert::Infallible;

use crate::{datagram::*, firmware::operation::ReadsFPGAStateOp};

use derive_more::Debug;

/// [`Datagram`] to enable reading the FPGA state.
#[derive(Debug)]
pub struct ReadsFPGAState<F: Fn(&Device) -> bool> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
}

impl<F: Fn(&Device) -> bool> ReadsFPGAState<F> {
    /// Creates a new [`ReadsFPGAState`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct ReadsFPGAStateOpGenerator<F: Fn(&Device) -> bool> {
    f: F,
}

impl<F: Fn(&Device) -> bool> OperationGenerator for ReadsFPGAStateOpGenerator<F> {
    type O1 = ReadsFPGAStateOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2 {})
    }
}

impl<F: Fn(&Device) -> bool> Datagram for ReadsFPGAState<F> {
    type G = ReadsFPGAStateOpGenerator<F>;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, Self::Error> {
        Ok(ReadsFPGAStateOpGenerator { f: self.f })
    }
}
