// GRCOV_EXCL_START

use std::convert::Infallible;

use crate::{
    datagram::*,
    firmware::{fpga::PulseWidth, operation::v10::PulseWidthEncoderOp},
};

use autd3_core::{common::DEFAULT_TIMEOUT, derive::DatagramOption, gain::EmitIntensity};
use derive_more::Debug;

const DEFAULT_TABLE: &[u8; 256] = include_bytes!("asin.dat");

fn default_table(i: EmitIntensity) -> PulseWidth<u8, 8> {
    PulseWidth::new(DEFAULT_TABLE[i.0 as usize]).unwrap()
}

/// [`Datagram`] to configure pulse width encoder table for v10 firmware.
///
/// The pulse width encoder table is a table to determine the pulse width (or duty ratio) from the intensity.
/// In the firmware, the intensity (0-255) is used as the index of the table to determine the pulse width (0-255).
/// The period of the ultrasound is mapped to 256, and therefore, the ultrasound output is the ultrasound is maximum when the pulse width is 128 (50% in duty ratio).
///
/// The default table is set by the arcsin function so that [`EmitIntensity`] is linear; that is, `table[i] = round(256*arcsin(i/255)/π)`.
///
/// # Example
///
/// To set the pulse width encoder table, please specify a function that takes the index of the table as an argument and returns the pulse width for each device.
///
/// The following example sets the pulse width to be linearly proportional to the intensity for all devices.
/// ```
/// # use autd3_driver::datagram::v10::PulseWidthEncoder;
/// # use autd3_driver::firmware::fpga::PulseWidth;
/// PulseWidthEncoder::new(|_dev| |i| PulseWidth::from_duty(i.0 as f32 / 255.).unwrap());
/// ```
///
/// [`EmitIntensity`]: crate::firmware::fpga::EmitIntensity
#[derive(Clone, Debug)]
pub struct PulseWidthEncoder<
    H: Fn(EmitIntensity) -> PulseWidth<u8, 8> + Send + Sync,
    F: Fn(&Device) -> H,
> {
    #[debug(ignore)]
    f: F,
}

impl<H: Fn(EmitIntensity) -> PulseWidth<u8, 8> + Send + Sync, F: Fn(&Device) -> H>
    PulseWidthEncoder<H, F>
{
    /// Creates a new [`PulseWidthEncoder`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl Default
    for PulseWidthEncoder<
        Box<dyn Fn(EmitIntensity) -> PulseWidth<u8, 8> + Send + Sync>,
        Box<dyn Fn(&Device) -> Box<dyn Fn(EmitIntensity) -> PulseWidth<u8, 8> + Send + Sync>>,
    >
{
    fn default() -> Self {
        Self::new(Box::new(|_| Box::new(default_table)))
    }
}

pub struct PulseWidthEncoderOpGenerator<
    H: Fn(EmitIntensity) -> PulseWidth<u8, 8> + Send + Sync,
    F: Fn(&Device) -> H,
> {
    f: F,
}

impl<H: Fn(EmitIntensity) -> PulseWidth<u8, 8> + Send + Sync, F: Fn(&Device) -> H>
    OperationGenerator for PulseWidthEncoderOpGenerator<H, F>
{
    type O1 = PulseWidthEncoderOp<H>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new((self.f)(device)), Self::O2 {}))
    }
}

impl<H: Fn(EmitIntensity) -> PulseWidth<u8, 8> + Send + Sync, F: Fn(&Device) -> H> Datagram
    for PulseWidthEncoder<H, F>
{
    type G = PulseWidthEncoderOpGenerator<H, F>;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: &DeviceFilter) -> Result<Self::G, Self::Error> {
        Ok(PulseWidthEncoderOpGenerator { f: self.f })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            timeout: DEFAULT_TIMEOUT,
            parallel_threshold: num_cpus::get(),
        }
    }
}
// GRCOV_EXCL_STOP
