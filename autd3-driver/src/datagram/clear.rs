use crate::{datagram::*, defined::DEFAULT_TIMEOUT};

#[derive(Default)]
pub struct Clear {}

impl Clear {
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct ClearOpGenerator {}

impl<'a> OperationGenerator<'a> for ClearOpGenerator {
    type O1 = crate::firmware::operation::ClearOp;
    type O2 = crate::firmware::operation::NullOp;

    fn generate(&'a self, _: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::default(), Self::O2::default()))
    }
}

impl<'a> Datagram<'a> for Clear {
    type O1 = crate::firmware::operation::ClearOp;
    type O2 = crate::firmware::operation::NullOp;
    type G = ClearOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &'a Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ClearOpGenerator {})
    }
}
