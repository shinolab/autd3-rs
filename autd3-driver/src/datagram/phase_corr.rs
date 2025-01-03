use crate::{
    datagram::*,
    firmware::{fpga::Phase, operation::PhaseCorrectionOp},
    geometry::{Device, Transducer},
};

use autd3_derive::Builder;
use derive_more::Debug;
use derive_new::new;

/// [`Datagram`] to apply phase correction.
///
/// The phase value set here is added to the phase value by [`Gain`], [`FociSTM`], and [`GainSTM`].
///
/// # Example
///
/// ```
/// # use autd3_driver::datagram::PhaseCorrection;
/// # use autd3_driver::derive::Phase;
/// PhaseCorrection::new(|_dev| |_tr| Phase::PI);
/// ```
#[derive(Builder, Debug, new)]
pub struct PhaseCorrection<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> {
    #[debug(ignore)]
    f: F,
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
        (Self::O1::new((self.f)(device)), Self::O2::new())
    }
}

impl<FT: Fn(&Transducer) -> Phase + Send + Sync, F: Fn(&Device) -> FT> Datagram
    for PhaseCorrection<FT, F>
{
    type G = PhaseCorrectionOpGenerator<FT, F>;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDDriverError> {
        Ok(Self::G { f: self.f })
    }
}
