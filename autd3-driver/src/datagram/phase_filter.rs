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

impl<'a, P: Into<Phase>, FT: Fn(&Transducer) -> P + 'a, F: Fn(&Device) -> FT + Send + Sync>
    Datagram<'a> for PhaseFilter<'a, P, FT, F>
{
    type O1 = crate::firmware::operation::PhaseFilterOp<P, FT>;
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
