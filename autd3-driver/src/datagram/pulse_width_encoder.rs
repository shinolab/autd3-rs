use std::convert::Infallible;

use crate::{
    datagram::*,
    firmware::{fpga::PWE_BUF_SIZE, operation::PulseWidthEncoderOp},
};

use autd3_core::{defined::DEFAULT_TIMEOUT, derive::DatagramOption};
use derive_more::Debug;
use derive_new::new;

const DEFAULT_TABLE: &[u8; PWE_BUF_SIZE] = include_bytes!("asin.dat");

fn default_table(i: u8) -> u8 {
    DEFAULT_TABLE[i as usize]
}

/// [`Datagram`] to configure pulse width encoder table.
///
/// The pulse width encoder table is a table to determine the pulse width (or duty ratio) from the instensity.
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
/// # use autd3_driver::datagram::PulseWidthEncoder;
/// PulseWidthEncoder::new(|_dev| |i| (i as f32 / 255. * 128.).round() as u8);
/// ```
///
/// [`EmitIntensity`]: crate::firmware::fpga::EmitIntensity
#[derive(Clone, Debug, new)]
pub struct PulseWidthEncoder<H: Fn(u8) -> u8 + Send + Sync, F: Fn(&Device) -> H> {
    #[debug(ignore)]
    f: F,
}

impl Default
    for PulseWidthEncoder<
        Box<dyn Fn(u8) -> u8 + Send + Sync>,
        Box<dyn Fn(&Device) -> Box<dyn Fn(u8) -> u8 + Send + Sync>>,
    >
{
    fn default() -> Self {
        Self::new(Box::new(|_| Box::new(default_table)))
    }
}

pub struct PulseWidthEncoderOpGenerator<H: Fn(u8) -> u8 + Send + Sync, F: Fn(&Device) -> H> {
    f: F,
}

impl<H: Fn(u8) -> u8 + Send + Sync, F: Fn(&Device) -> H> OperationGenerator
    for PulseWidthEncoderOpGenerator<H, F>
{
    type O1 = PulseWidthEncoderOp<H>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2 {})
    }
}

impl<H: Fn(u8) -> u8 + Send + Sync, F: Fn(&Device) -> H> Datagram for PulseWidthEncoder<H, F> {
    type G = PulseWidthEncoderOpGenerator<H, F>;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: &DatagramOption) -> Result<Self::G, Self::Error> {
        Ok(PulseWidthEncoderOpGenerator { f: self.f })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            timeout: DEFAULT_TIMEOUT,
            parallel_threshold: 4,
        }
    }
}
