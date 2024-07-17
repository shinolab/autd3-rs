use crate::firmware::{fpga::PWE_BUF_SIZE, operation::PulseWidthEncoderOp};

use itertools::Itertools;

use crate::datagram::*;

const DEFAULT_TABLE: &[u8; PWE_BUF_SIZE] = include_bytes!("asin.dat");

fn default_table(i: u8) -> u8 {
    DEFAULT_TABLE[i as usize]
}

#[derive(Debug, Clone)]
pub struct PulseWidthEncoder<H: Fn(u8) -> u8 + Send + Sync, F: Fn(&Device) -> H> {
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
        PulseWidthEncoder::new(Box::new(|_| Box::new(default_table)))
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

    #[tracing::instrument(skip(self, geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());

        if tracing::enabled!(tracing::Level::DEBUG) {
            geometry.devices().for_each(|dev| {
                let f = (self.f)(dev);
                if tracing::enabled!(tracing::Level::TRACE) {
                    tracing::debug!(
                        "Device[{}]: {}",
                        dev.idx(),
                        (0..PWE_BUF_SIZE)
                            .map(|i| f(i as u8))
                            .format_with(", ", |elt, f| f(&format_args!("{:#04X}", elt)))
                    );
                } else {
                    tracing::debug!(
                        "Device[{}]: {:#04X}, ..., {:#04X}",
                        dev.idx(),
                        f(0),
                        f((PWE_BUF_SIZE - 1) as u8)
                    );
                }
            });
        }
    }
    // GRCOV_EXCL_STOP
}
