use crate::{
    datagram::*,
    derive::*,
    firmware::{fpga::Phase, operation::PhaseCorrectionOp},
    geometry::Transducer,
};

use derive_more::Debug;

#[derive(Builder, Debug)]
pub struct PhaseCorrection<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> {
    #[debug(ignore)]
    #[get(ref)]
    f: F,
}

impl<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> PhaseCorrection<FT, F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct PhaseCorrectionOpGenerator<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> {
    f: F,
}

impl<FT: Fn(&Transducer) -> Phase + Send + Sync, F: Fn(&Device) -> FT> OperationGenerator
    for PhaseCorrectionOpGenerator<FT, F>
{
    type O1 = PhaseCorrectionOp<FT>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::default())
    }
}

impl<FT: Fn(&Transducer) -> Phase + Send + Sync, F: Fn(&Device) -> FT> Datagram
    for PhaseCorrection<FT, F>
{
    type G = PhaseCorrectionOpGenerator<FT, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(Self::G { f: self.f })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}
