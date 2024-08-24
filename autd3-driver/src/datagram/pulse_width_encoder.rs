use crate::{
    datagram::*,
    firmware::{fpga::PWE_BUF_SIZE, operation::PulseWidthEncoderOp},
};

use derive_more::Debug;

const DEFAULT_TABLE: &[u8; PWE_BUF_SIZE] = include_bytes!("asin.dat");

fn default_table(i: u8) -> u8 {
    DEFAULT_TABLE[i as usize]
}

#[derive(Clone, Debug)]
pub struct PulseWidthEncoder<H: Fn(u8) -> u8 + Send + Sync, F: Fn(&Device) -> H> {
    #[debug(ignore)]
    f: F,
}

impl<H: Fn(u8) -> u8 + Send + Sync, F: Fn(&Device) -> H> PulseWidthEncoder<H, F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
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

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::default())
    }
}

impl<H: Fn(u8) -> u8 + Send + Sync, F: Fn(&Device) -> H> Datagram for PulseWidthEncoder<H, F> {
    type G = PulseWidthEncoderOpGenerator<H, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(PulseWidthEncoderOpGenerator { f: self.f })
    }
}
