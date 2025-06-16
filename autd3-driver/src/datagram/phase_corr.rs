use std::convert::Infallible;

use crate::geometry::{Device, Transducer};

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    derive::FirmwareLimits,
    gain::Phase,
    geometry::Geometry,
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
/// # use autd3_core::gain::Phase;
/// PhaseCorrection::new(|_dev| |_tr| Phase::PI);
/// ```
///
/// [`Gain`]: autd3_core::gain::Gain
/// [`FociSTM`]: crate::datagram::FociSTM
/// [`GainSTM`]: crate::datagram::GainSTM
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

impl<FT: Fn(&Transducer) -> Phase + Send + Sync, F: Fn(&Device) -> FT> Datagram
    for PhaseCorrection<FT, F>
{
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &DeviceFilter,
        _: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
