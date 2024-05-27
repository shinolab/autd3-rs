use crate::{
    datagram::*,
    defined::DEFAULT_TIMEOUT,
    derive::{Device, Transducer},
    firmware::fpga::Phase,
};

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
    > OperationGenerator<'a> for PhaseFilterOpGenerator<P, FT, F>
{
    type O1 = crate::firmware::operation::PhaseFilterOp<P, FT>;
    type O2 = crate::firmware::operation::NullOp;

    fn generate(&'a self, device: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new((self.f)(device)), Self::O2::default()))
    }
}

impl<
        'a,
        P: Into<Phase>,
        FT: Fn(&Transducer) -> P + Send + Sync + 'a,
        F: Fn(&Device) -> FT + Send + Sync + 'a,
    > Datagram<'a> for PhaseFilter<'a, P, FT, F>
{
    type O1 = crate::firmware::operation::PhaseFilterOp<P, FT>;
    type O2 = crate::firmware::operation::NullOp;
    type G =  PhaseFilterOpGenerator<P, FT, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &'a Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(PhaseFilterOpGenerator { f: self.f })
    }
}
