use crate::firmware::operation::ClearOp;

use crate::datagram::*;

#[derive(Default, Debug)]
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

impl Datagram for Clear {
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

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::info!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}
