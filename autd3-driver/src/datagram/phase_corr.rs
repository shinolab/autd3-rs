use std::convert::Infallible;

use crate::{
    datagram::*,
    firmware::{fpga::Phase, operation::PhaseCorrectionOp},
    geometry::{Device, Transducer},
};

use derive_more::Debug;

/// [`Datagram`] to apply phase correction.
///
/// The phase value set here is added to the phase value by [`Gain`], [`FociSTM`], and [`GainSTM`].
///
/// # Example
///
/// ```
/// # use autd3_driver::datagram::PhaseCorrection;
/// # use autd3_driver::firmware::fpga::Phase;
/// PhaseCorrection::new(|_dev| |_tr| Phase::PI);
/// ```
///
/// [`Gain`]: autd3_core::gain::Gain
#[derive(Debug)]
pub struct PhaseCorrection<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
}

impl<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> PhaseCorrection<FT, F> {
    /// Creates a new [`PhaseCorrection`].
    #[must_use]
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
        (Self::O1::new((self.f)(device)), Self::O2 {})
    }
}

impl<FT: Fn(&Transducer) -> Phase + Send + Sync, F: Fn(&Device) -> FT> Datagram
    for PhaseCorrection<FT, F>
{
    type G = PhaseCorrectionOpGenerator<FT, F>;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: bool) -> Result<Self::G, Self::Error> {
        Ok(Self::G { f: self.f })
    }
}
