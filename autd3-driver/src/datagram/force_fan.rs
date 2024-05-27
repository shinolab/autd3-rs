use crate::{datagram::*, derive::DEFAULT_TIMEOUT, geometry::Device};

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

impl<'a, F: Fn(&Device) -> bool + Send + Sync> OperationGenerator<'a> for ForceFanOpGenerator<F> {
    type O1 = crate::firmware::operation::ForceFanOp;
    type O2 = crate::firmware::operation::NullOp;

    fn generate(&'a self, device: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new((self.f)(device)), Self::O2::default()))
    }
}

impl<'a, F: Fn(&Device) -> bool + Send + Sync + 'a> Datagram<'a> for ForceFan<F> {
    type O1 = crate::firmware::operation::ForceFanOp;
    type O2 = crate::firmware::operation::NullOp;
    type G = ForceFanOpGenerator<F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &'a Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ForceFanOpGenerator { f: self.f })
    }
}
