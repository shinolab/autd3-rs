use std::{convert::Infallible, f32::consts::PI};

use autd3_core::{
    common::DEFAULT_TIMEOUT,
    datagram::{Datagram, DatagramOption, DeviceFilter, PulseWidth},
    derive::FirmwareLimits,
    environment::Environment,
    gain::Intensity,
    geometry::{Device, Geometry},
};

use derive_more::Debug;
use num::Zero;

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
/// # use autd3_core::datagram::PulseWidth;
/// // For firmware version 11 or later
/// PulseWidthEncoder::new(|_dev| |i| PulseWidth::<9, u16>::from_duty(i.0 as f32 / 510.).unwrap());
/// // For firmware version 10
/// PulseWidthEncoder::new(|_dev| |i| PulseWidth::<8, u8>::from_duty(i.0 as f32 / 510.).unwrap());
/// ```
///
/// [`Intensity`]: autd3_core::gain::Intensity
#[derive(Clone, Debug)]
pub struct PulseWidthEncoder<P, F> {
    #[debug(ignore)]
    pub(crate) f: F,
    phantom: std::marker::PhantomData<P>,
}

impl<
    const BITS: usize,
    T: Copy + TryFrom<usize> + Zero + PartialOrd,
    H: Fn(Intensity) -> PulseWidth<BITS, T> + Send + Sync,
    F: Fn(&Device) -> H,
> PulseWidthEncoder<PulseWidth<BITS, T>, F>
{
    /// Creates a new [`PulseWidthEncoder`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self {
            f,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<
    const BITS: usize,
    T: Copy + TryFrom<usize> + Zero + PartialOrd,
    H: Fn(Intensity) -> PulseWidth<BITS, T> + Send + Sync,
    F: Fn(&Device) -> H,
> Datagram for PulseWidthEncoder<PulseWidth<BITS, T>, F>
{
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

    fn option(&self) -> DatagramOption {
        DatagramOption {
            timeout: DEFAULT_TIMEOUT,
            parallel_threshold: num_cpus::get(),
        }
    }
}

impl<const BITS: usize, T: Copy + TryFrom<usize> + Zero + PartialOrd> Default
    for PulseWidthEncoder<PulseWidth<BITS, T>, fn(&Device) -> fn(Intensity) -> PulseWidth<BITS, T>>
{
    fn default() -> Self {
        Self::new(|_| {
            |intensity| PulseWidth::from_duty((intensity.0 as f32 / 255.).asin() / PI).unwrap()
        })
    }
}
