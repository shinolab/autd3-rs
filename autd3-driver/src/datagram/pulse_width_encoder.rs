use crate::{datagram::*, defined::DEFAULT_TIMEOUT, firmware::fpga::PULSE_WIDTH_MAX};

const DEFAULT_TABLE: &[u8; 65536] = include_bytes!("asin.dat");

fn default_table(i: usize) -> u16 {
    if i >= 0xFF * 0xFF {
        PULSE_WIDTH_MAX
    } else {
        DEFAULT_TABLE[i] as u16
    }
}

#[derive(Debug, Clone)]
pub struct PulseWidthEncoder<H: Fn(usize) -> u16, F: Fn(&Device) -> H + Send + Sync> {
    f: F,
}

impl<H: Fn(usize) -> u16, F: Fn(&Device) -> H + Send + Sync> PulseWidthEncoder<H, F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl Default for PulseWidthEncoder<fn(usize) -> u16, fn(&Device) -> fn(usize) -> u16> {
    fn default() -> Self {
        PulseWidthEncoder::new(|_| default_table)
    }
}

pub struct PulseWidthEncoderOpGenerator<H: Fn(usize) -> u16, F: Fn(&Device) -> H + Send + Sync> {
    f: F,
}

impl<'a, H: Fn(usize) -> u16 + 'a, F: Fn(&Device) -> H + Send + Sync> OperationGenerator<'a>
    for PulseWidthEncoderOpGenerator<H, F>
{
    type O1 = crate::firmware::operation::PulseWidthEncoderOp<H>;
    type O2 = crate::firmware::operation::NullOp;

    fn generate(&'a self, device: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new((self.f)(device)), Self::O2::default()))
    }
}

impl<'a, H: Fn(usize) -> u16 + 'a, F: Fn(&Device) -> H + Send + Sync + 'a> Datagram<'a>
    for PulseWidthEncoder<H, F>
{
    type O1 = crate::firmware::operation::PulseWidthEncoderOp<H>;
    type O2 = crate::firmware::operation::NullOp;
    type G =  PulseWidthEncoderOpGenerator<H, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &'a Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(PulseWidthEncoderOpGenerator { f: self.f })
    }
}
