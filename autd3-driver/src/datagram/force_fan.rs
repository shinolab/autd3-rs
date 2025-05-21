use std::convert::Infallible;

use crate::{datagram::*, firmware::operation::ForceFanOp};

use derive_more::Debug;

/// [`Datagram`] to force the fan to run.
#[derive(Debug)]
pub struct ForceFan<F: Fn(&Device) -> bool> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
}

impl<F: Fn(&Device) -> bool> ForceFan<F> {
    /// Creates a new [`ForceFan`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct ForceFanOpGenerator<F: Fn(&Device) -> bool> {
    f: F,
}

impl<F: Fn(&Device) -> bool> OperationGenerator for ForceFanOpGenerator<F> {
    type O1 = ForceFanOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2 {})
    }
}

impl<F: Fn(&Device) -> bool> Datagram for ForceFan<F> {
    type G = ForceFanOpGenerator<F>;
    type Error = Infallible;

    fn operation_generator(self, _: &mut Geometry) -> Result<Self::G, Self::Error> {
        Ok(ForceFanOpGenerator { f: self.f })
    }
}
