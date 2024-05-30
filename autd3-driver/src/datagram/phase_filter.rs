use crate::{
    derive::{Phase, Transducer},
    firmware::operation::PhaseFilterOp,
};

use crate::datagram::*;

#[derive(Debug, Clone, Copy)]
pub struct PhaseFilter<P: Into<Phase>, FT: Fn(&Transducer) -> P, F: Fn(&Device) -> FT> {
    f: F,
}

impl<P: Into<Phase>, FT: Fn(&Transducer) -> P, F: Fn(&Device) -> FT> PhaseFilter<P, FT, F> {
    pub const fn additive(f: F) -> Self {
        Self { f }
    }
}

pub struct PhaseFilterOpGenerator<
    P: Into<Phase>,
    FT: Fn(&Transducer) -> P + Send + Sync,
    F: Fn(&Device) -> FT,
> {
    f: F,
}

impl<P: Into<Phase>, FT: Fn(&Transducer) -> P + Send + Sync, F: Fn(&Device) -> FT>
    OperationGenerator for PhaseFilterOpGenerator<P, FT, F>
{
    type O1 = PhaseFilterOp<P, FT>;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::default())
    }
}

impl<P: Into<Phase>, FT: Fn(&Transducer) -> P + Send + Sync, F: Fn(&Device) -> FT> Datagram
    for PhaseFilter<P, FT, F>
{
    type O1 = PhaseFilterOp<P, FT>;
    type O2 = NullOp;
    type G = PhaseFilterOpGenerator<P, FT, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(PhaseFilterOpGenerator { f: self.f })
    }
}

#[cfg(feature = "capi")]
impl Default
    for PhaseFilter<
        Phase,
        Box<dyn Fn(&Transducer) -> Phase + Send + Sync>,
        Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Phase + Send + Sync>>,
    >
{
    fn default() -> Self {
        Self::additive(Box::new(|_| Box::new(|_| Phase::new(0))))
    }
}
