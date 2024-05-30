use crate::{datagram::*, derive::*, firmware::operation::ReadsFPGAStateOp};

#[derive(Builder)]
pub struct ReadsFPGAState<F: Fn(&Device) -> bool> {
    #[get]
    f: F,
}

impl<F: Fn(&Device) -> bool> ReadsFPGAState<F> {
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

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::default())
    }
}

impl<F: Fn(&Device) -> bool> Datagram for ReadsFPGAState<F> {
    type O1 = ReadsFPGAStateOp;
    type O2 = NullOp;
    type G = ReadsFPGAStateOpGenerator<F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ReadsFPGAStateOpGenerator { f: self.f })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}
