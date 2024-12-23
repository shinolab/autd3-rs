use crate::{datagram::*, firmware::operation::ForceFanOp};

use autd3_derive::Builder;
use derive_more::Debug;
use derive_new::new;

/// [`Datagram`] to force the fan to run.
#[derive(Builder, Debug, new)]
pub struct ForceFan<F: Fn(&Device) -> bool> {
    #[debug(ignore)]
    #[get(ref, no_doc)]
    f: F,
}

pub struct ForceFanOpGenerator<F: Fn(&Device) -> bool> {
    f: F,
}

impl<F: Fn(&Device) -> bool> OperationGenerator for ForceFanOpGenerator<F> {
    type O1 = ForceFanOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::new())
    }
}

impl<F: Fn(&Device) -> bool> Datagram for ForceFan<F> {
    type G = ForceFanOpGenerator<F>;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDDriverError> {
        Ok(ForceFanOpGenerator { f: self.f })
    }
}
