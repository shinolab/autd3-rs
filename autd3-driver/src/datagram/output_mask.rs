use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceFilter, FirmwareLimits, Segment},
    environment::Environment,
    geometry::{Device, Geometry, Transducer},
};

use derive_more::Debug;

/// [`Datagram`] to mask output.
///
/// The transducers set to `false` in [`OutputMask`] will not output regardless of the intensity settings by [`Gain`], [`FociSTM`], and [`GainSTM`].
///
/// # Example
///
/// ```
/// # use autd3_core::datagram::Segment;
/// # use autd3_driver::datagram::OutputMask;
/// OutputMask::new(|_dev| |_tr| true, Segment::S0);
/// ```
///
/// [`Gain`]: autd3_core::gain::Gain
/// [`FociSTM`]: crate::datagram::FociSTM
/// [`GainSTM`]: crate::datagram::GainSTM
#[derive(Debug)]
pub struct OutputMask<F> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
    #[doc(hidden)]
    pub segment: Segment,
}

impl<FT: Fn(&Transducer) -> bool, F: Fn(&Device) -> FT> OutputMask<F> {
    /// Creates a new [`OutputMask`].
    #[must_use]
    pub const fn new(f: F, segment: Segment) -> Self {
        Self { f, segment }
    }
}

impl<FT: Fn(&Transducer) -> bool + Send + Sync, F: Fn(&Device) -> FT> Datagram for OutputMask<F> {
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &Environment,
        _: &DeviceFilter,
        _: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
