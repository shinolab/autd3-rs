use crate::firmware::operation::ClearOp;

use crate::datagram::*;

#[derive(Default)]
pub struct Clear {}

impl Clear {
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct ClearOpGenerator {}

impl OperationGenerator for ClearOpGenerator {
    type O1 = ClearOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::default(), Self::O2::default())
    }
}

impl<'a> Datagram<'a> for Clear {
    type O1 = ClearOp;
    type O2 = NullOp;
    type G = ClearOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ClearOpGenerator {})
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}
