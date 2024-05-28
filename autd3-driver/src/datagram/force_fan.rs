use crate::firmware::operation::ForceFanOp;

use crate::datagram::*;

pub struct ForceFan<F: Fn(&Device) -> bool + Send + Sync> {
    f: F,
}

impl<F: Fn(&Device) -> bool + Send + Sync> ForceFan<F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct ForceFanOpGenerator<F: Fn(&Device) -> bool + Send + Sync> {
    f: F,
}

impl<F: Fn(&Device) -> bool + Send + Sync> OperationGenerator for ForceFanOpGenerator<F> {
    type O1 = ForceFanOp;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new((self.f)(device)), Self::O2::default()))
    }
}

impl<'a, F: Fn(&Device) -> bool + Send + Sync + 'a> Datagram<'a> for ForceFan<F> {
    type O1 = ForceFanOp;
    type O2 = NullOp;
    type G = ForceFanOpGenerator<F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ForceFanOpGenerator { f: self.f })
    }
}
