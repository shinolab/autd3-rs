use crate::{datagram::*, defined::DEFAULT_TIMEOUT, geometry::Device};

pub struct ReadsFPGAState<F: Fn(&Device) -> bool + Send + Sync> {
    f: F,
}

impl<F: Fn(&Device) -> bool + Send + Sync> ReadsFPGAState<F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<'a, F: Fn(&Device) -> bool + Send + Sync> Datagram<'a> for ReadsFPGAState<F> {
    type O1 = crate::firmware::operation::ReadsFPGAStateOp;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(
        &'a self,
        _: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        let f = &self.f;
        Ok(|dev| (Self::O1::new(f(dev)), Self::O2::default()))
    }
}
