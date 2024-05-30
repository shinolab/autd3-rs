use crate::firmware::{fpga::PULSE_WIDTH_MAX, operation::PulseWidthEncoderOp};

use crate::datagram::*;

const DEFAULT_TABLE: &[u8; 65536] = include_bytes!("asin.dat");

fn default_table(i: usize) -> u16 {
    if i >= 0xFF * 0xFF {
        PULSE_WIDTH_MAX
    } else {
        DEFAULT_TABLE[i] as u16
    }
}

#[derive(Debug, Clone)]
pub struct PulseWidthEncoder<H: Fn(usize) -> u16 + Send + Sync, F: Fn(&Device) -> H> {
    f: F,
}

impl<H: Fn(usize) -> u16 + Send + Sync, F: Fn(&Device) -> H> PulseWidthEncoder<H, F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl Default
    for PulseWidthEncoder<
        Box<dyn Fn(usize) -> u16 + Send + Sync>,
        Box<dyn Fn(&Device) -> Box<dyn Fn(usize) -> u16 + Send + Sync>>,
    >
{
    fn default() -> Self {
        PulseWidthEncoder::new(Box::new(|_| Box::new(default_table)))
    }
}

pub struct PulseWidthEncoderOpGenerator<H: Fn(usize) -> u16 + Send + Sync, F: Fn(&Device) -> H> {
    f: F,
}

impl<H: Fn(usize) -> u16 + Send + Sync, F: Fn(&Device) -> H> OperationGenerator
    for PulseWidthEncoderOpGenerator<H, F>
{
    type O1 = PulseWidthEncoderOp<H>;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::default())
    }
}

impl<H: Fn(usize) -> u16 + Send + Sync, F: Fn(&Device) -> H> Datagram for PulseWidthEncoder<H, F> {
    type O1 = PulseWidthEncoderOp<H>;
    type O2 = NullOp;
    type G = PulseWidthEncoderOpGenerator<H, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(PulseWidthEncoderOpGenerator { f: self.f })
    }
}
