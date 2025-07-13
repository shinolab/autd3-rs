use std::convert::Infallible;

use autd3_core::{
    datagram::{DatagramOption, DatagramS, DeviceFilter, FirmwareLimits, Segment},
    environment::Environment,
    geometry::{Device, Geometry, Transducer},
};

use derive_more::Debug;

/// [`Datagram`] to mask output.
///
/// The transducers set to `false` in [`OutputMask`] will not output ultrasound regardless of the intensity settings by [`Gain`], [`FociSTM`], and [`GainSTM`].
///
/// # Example
///
/// ```
/// # use autd3_driver::datagram::OutputMask;
/// OutputMask::new(|_dev| |_tr| true);
/// ```
///
/// [`Datagram`]: autd3_core::datagram::Datagram
/// [`Gain`]: autd3_core::gain::Gain
/// [`FociSTM`]: crate::datagram::FociSTM
/// [`GainSTM`]: crate::datagram::GainSTM
#[derive(Debug)]
pub struct OutputMask<F, FT> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
    _phantom: std::marker::PhantomData<FT>,
}

impl<'dev, 'tr, FT: Fn(&'tr Transducer) -> bool, F: Fn(&'dev Device) -> FT> OutputMask<F, FT>
where
    'dev: 'tr,
{
    /// Creates a new [`OutputMask`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct OutputMaskOperationGenerator<F> {
    pub(crate) f: F,
    pub(crate) segment: Segment,
}

impl<'geo, 'dev, 'tr, FT: Fn(&'tr Transducer) -> bool + Send + Sync, F: Fn(&'dev Device) -> FT>
    DatagramS<'geo, 'dev, 'tr> for OutputMask<F, FT>
{
    type G = OutputMaskOperationGenerator<F>;
    type Error = Infallible;

    fn operation_generator_with_segment(
        self,
        _: &'geo Geometry,
        _: &Environment,
        _: &DeviceFilter,
        _: &FirmwareLimits,
        segment: Segment,
        _: Option<autd3_core::derive::TransitionMode>,
    ) -> Result<Self::G, Self::Error> {
        Ok(OutputMaskOperationGenerator { f: self.f, segment })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption::default()
    }
}
