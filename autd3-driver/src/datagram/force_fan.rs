use std::convert::Infallible;

use crate::{datagram::*, firmware::operation::ForceFanOp};

use autd3_core::datagram::DatagramOption;
use derive_more::Debug;
use derive_new::new;

/// [`Datagram`] to force the fan to run.
#[derive(Debug, new)]
pub struct ForceFan<F: Fn(&Device) -> bool> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
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

    fn operation_generator(self, _: &Geometry, _: &DatagramOption) -> Result<Self::G, Self::Error> {
        Ok(ForceFanOpGenerator { f: self.f })
    }
}
