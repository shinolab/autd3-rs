use crate::{datagram::*, defined::DEFAULT_TIMEOUT};

#[derive(Default)]
pub struct Clear {}

impl Clear {
    pub const fn new() -> Self {
        Self {}
    }
}

impl<'a> Datagram<'a> for Clear {
    type O1 = crate::firmware::operation::ClearOp;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(
        &'a self,
        _: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2), AUTDInternalError> {
        Ok(move |_| (Self::O1::default(), Self::O2::default()))
    }
}
