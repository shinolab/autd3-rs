use std::{convert::Infallible, mem::size_of};

use super::{
    super::fpga::{PWE_BUF_SIZE, ULTRASOUND_PERIOD_COUNT_BITS},
    Operation, OperationGenerator,
};
use crate::{datagram::PulseWidthEncoder, firmware::tag::TypeTag};

use autd3_core::{datagram::PulseWidth, gain::Intensity, geometry::Device};

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct PweMsg {
    tag: TypeTag,
    __: u8,
}

pub struct PulseWidthEncoderOp<F: Fn(Intensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u16>> {
    is_done: bool,
    f: F,
}

impl<F: Fn(Intensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u16>> PulseWidthEncoderOp<F> {
    pub(crate) const fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<F: Fn(Intensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u16> + Send + Sync> Operation
    for PulseWidthEncoderOp<F>
{
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
            tx,
            PweMsg {
                tag: TypeTag::ConfigPulseWidthEncoderV11,
                __: 0,
            },
        );

        tx[size_of::<PweMsg>()..]
            .chunks_mut(size_of::<u16>())
            .take(PWE_BUF_SIZE)
            .enumerate()
            .for_each(|(i, dst)| {
                crate::firmware::driver::write_to_tx(
                    dst,
                    (self.f)(Intensity(i as u8)).pulse_width(),
                );
            });

        self.is_done = true;

        Ok(size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<
    H: Fn(Intensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u16> + Send + Sync,
    F: Fn(&Device) -> H,
> OperationGenerator for PulseWidthEncoder<PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u16>, F>
{
    type O1 = PulseWidthEncoderOp<H>;
    type O2 = crate::firmware::driver::NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new((self.f)(device)), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test() {
        let device = crate::autd3_device::tests::create_device();

        let mut tx = [0x00u8; 2 * (size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u16>())];

        let mut op = PulseWidthEncoderOp::new(|i| PulseWidth::new(i.0 as u16).unwrap());

        assert_eq!(
            size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>(),
            op.required_size(&device)
        );

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::ConfigPulseWidthEncoderV11 as u8);
        assert!((0..PWE_BUF_SIZE).all(|i| i as u8 == tx[2 + 2 * i]));
    }
}
