use crate::{datagram::*, derive::*, firmware::operation::ForceFanOp};

use derive_more::Debug;
use derive_new::new;

#[derive(Builder, Debug, new)]
pub struct ForceFan<F: Fn(&Device) -> bool> {
    #[debug(ignore)]
    #[get(ref)]
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

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ForceFanOpGenerator { f: self.f })
    }
}
