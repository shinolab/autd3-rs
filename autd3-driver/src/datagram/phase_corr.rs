use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceFilter, FirmwareLimits},
    environment::Environment,
    gain::Phase,
    geometry::{Device, Geometry, Transducer},
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
pub struct PhaseCorrection<F, FT> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
    _phantom: std::marker::PhantomData<FT>,
}

impl<'a, FT: Fn(&'a Transducer) -> Phase, F: Fn(&'a Device) -> FT> PhaseCorrection<F, FT> {
    /// Creates a new [`PhaseCorrection`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, FT: Fn(&'a Transducer) -> Phase + Send + Sync, F: Fn(&'a Device) -> FT> Datagram<'a>
    for PhaseCorrection<F, FT>
{
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &'a Geometry,
        _: &Environment,
        _: &DeviceFilter,
        _: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
