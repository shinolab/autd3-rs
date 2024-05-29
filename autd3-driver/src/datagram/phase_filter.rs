use crate::{
    derive::{Phase, Transducer},
    firmware::operation::PhaseFilterOp,
};

use crate::datagram::*;

#[derive(Debug, Clone, Copy)]
pub struct PhaseFilter<
    'a,
    P: Into<Phase>,
    FT: Fn(&Transducer) -> P + 'a,
    F: Fn(&Device) -> FT + Send + Sync,
> {
    f: F,
    _phantom: std::marker::PhantomData<&'a P>,
}

impl<'a, P: Into<Phase>, FT: Fn(&Transducer) -> P + 'a, F: Fn(&Device) -> FT + Send + Sync>
    PhaseFilter<'a, P, FT, F>
{
    pub const fn additive(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct PhaseFilterOpGenerator<
    P: Into<Phase>,
    FT: Fn(&Transducer) -> P + Send + Sync,
    F: Fn(&Device) -> FT + Send + Sync,
> {
    f: F,
}

impl<
        'a,
        P: Into<Phase> + 'a,
        FT: Fn(&Transducer) -> P + Send + Sync + 'a,
        F: Fn(&Device) -> FT + Send + Sync + 'a,
    > OperationGenerator for PhaseFilterOpGenerator<P, FT, F>
{
    type O1 = PhaseFilterOp<P, FT>;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::default())
    }
}

impl<
        'a,
        P: Into<Phase>,
        FT: Fn(&Transducer) -> P + Send + Sync + 'a,
        F: Fn(&Device) -> FT + Send + Sync + 'a,
    > Datagram<'a> for PhaseFilter<'a, P, FT, F>
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
