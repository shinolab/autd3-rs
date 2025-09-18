use std::{convert::Infallible, f32::consts::PI};

use autd3_core::{
    common::DEFAULT_TIMEOUT,
    datagram::{Datagram, DatagramOption, DeviceMask},
    environment::Environment,
    firmware::{FirmwareLimits, Intensity, PulseWidth},
    geometry::{Device, Geometry},
};

/// [`Datagram`] to configure pulse width encoder table.
///
/// The pulse width encoder table is a table to determine the pulse width (or duty ratio) from the intensity.
/// In the firmware, the intensity (0-255) is used as the index of the table to determine the pulse width.
/// For firmware v11 or later, the period of the ultrasound is mapped to 512, therefore, the ultrasound output becomes maximum when the pulse width is 256 (50% in duty ratio).
/// For firmware v10, the period of the ultrasound is mapped to 256, therefore, the ultrasound output becomes maximum when the pulse width is 128 (50% in duty ratio).
///
/// The default table is set by the arcsin function so that [`Intensity`] is linearly proportional to output ultrasound pressure; that is, `table[i] = round(T*arcsin(i/255)/π)` where `T` is the period of the ultrasound.
///
/// # Example
///
/// To set the pulse width encoder table, please specify a function that takes the index of the table as an argument and returns the pulse width for each device.
///
/// The following example sets the pulse width to be linearly proportional to the intensity for all devices.
/// ```
/// # use autd3_driver::datagram::PulseWidthEncoder;
/// # use autd3_core::firmware::PulseWidth;
/// PulseWidthEncoder::new(|_dev| |i| PulseWidth::from_duty(i.0 as f32 / 510.).unwrap());
/// ```
///
/// [`Intensity`]: autd3_core::firmware::Intensity
#[derive(Clone, Debug)]
pub struct PulseWidthEncoder<F> {
    pub(crate) f: F,
}

impl<H: Fn(Intensity) -> PulseWidth + Send + Sync, F: Fn(&Device) -> H> PulseWidthEncoder<F> {
    /// Creates a new [`PulseWidthEncoder`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<H: Fn(Intensity) -> PulseWidth + Send + Sync, F: Fn(&Device) -> H> Datagram<'_>
    for PulseWidthEncoder<F>
{
    type G = PulseWidthEncoderOperationGenerator<F>;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &Environment,
        _: &DeviceMask,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(Self::G {
            f: self.f,
            limits: *limits,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            timeout: DEFAULT_TIMEOUT,
            parallel_threshold: std::thread::available_parallelism()
                .map(std::num::NonZeroUsize::get)
                .unwrap_or(8),
        }
    }
}

impl Default for PulseWidthEncoder<fn(&Device) -> fn(Intensity) -> PulseWidth> {
    fn default() -> Self {
        Self::new(|_| {
            |intensity| PulseWidth::from_duty((intensity.0 as f32 / 255.).asin() / PI).unwrap()
        })
    }
}

#[doc(hidden)]
pub struct PulseWidthEncoderOperationGenerator<F> {
    pub f: F,
    pub limits: FirmwareLimits,
}
