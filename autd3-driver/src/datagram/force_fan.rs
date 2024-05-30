use crate::{datagram::*, derive::*, firmware::operation::ForceFanOp};

#[derive(Builder)]
pub struct ForceFan<F: Fn(&Device) -> bool> {
    #[get]
    f: F,
}

impl<F: Fn(&Device) -> bool> ForceFan<F> {
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

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::default())
    }
}

impl<F: Fn(&Device) -> bool> Datagram for ForceFan<F> {
    type O1 = ForceFanOp;
    type O2 = NullOp;
    type G = ForceFanOpGenerator<F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ForceFanOpGenerator { f: self.f })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}

#[cfg(feature = "capi")]
impl Default for ForceFan<Box<dyn Fn(&Device) -> bool>> {
    fn default() -> Self {
        Self::new(Box::new(|_| false))
    }
}
