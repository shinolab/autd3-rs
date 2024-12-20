use crate::{datagram::*, firmware::operation::ReadsFPGAStateOp};

use autd3_derive::Builder;
use derive_more::Debug;
use derive_new::new;

#[derive(Builder, Debug, new)]
pub struct ReadsFPGAState<F: Fn(&Device) -> bool> {
    #[debug(ignore)]
    #[get(ref)]
    f: F,
}

pub struct ReadsFPGAStateOpGenerator<F: Fn(&Device) -> bool> {
    f: F,
}

impl<F: Fn(&Device) -> bool> OperationGenerator for ReadsFPGAStateOpGenerator<F> {
    type O1 = ReadsFPGAStateOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::new())
    }
}

impl<F: Fn(&Device) -> bool> Datagram for ReadsFPGAState<F> {
    type G = ReadsFPGAStateOpGenerator<F>;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDDriverError> {
        Ok(ReadsFPGAStateOpGenerator { f: self.f })
    }
}
