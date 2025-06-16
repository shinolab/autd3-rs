use std::{convert::Infallible, mem::size_of};

use super::{
    super::fpga::{PWE_BUF_SIZE, ULTRASOUND_PERIOD_COUNT_BITS},
    Operation, OperationGenerator,
    null::NullOp,
};
use crate::{datagram::PulseWidthEncoder, firmware::tag::TypeTag, geometry::Device};

use autd3_core::{datagram::PulseWidth, gain::EmitIntensity};

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct PweMsg {
    tag: TypeTag,
    __: u8,
}

pub struct PulseWidthEncoderOp<F: Fn(EmitIntensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u8>>
{
    is_done: bool,
    f: F,
}

impl<F: Fn(EmitIntensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u8>> PulseWidthEncoderOp<F> {
    pub(crate) const fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<F: Fn(EmitIntensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u8> + Send + Sync> Operation
    for PulseWidthEncoderOp<F>
{
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        super::write_to_tx(
            tx,
            PweMsg {
                tag: TypeTag::ConfigPulseWidthEncoderV10,
                __: 0,
            },
        );

        tx[size_of::<PweMsg>()..]
            .iter_mut()
            .take(PWE_BUF_SIZE)
            .enumerate()
            .for_each(|(i, dst)| {
                *dst = (self.f)(EmitIntensity(i as u8)).pulse_width();
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
    H: Fn(EmitIntensity) -> PulseWidth<ULTRASOUND_PERIOD_COUNT_BITS, u8> + Send + Sync,
    F: Fn(&Device) -> H,
> OperationGenerator for PulseWidthEncoder<ULTRASOUND_PERIOD_COUNT_BITS, u8, H, F>
{
    type O1 = PulseWidthEncoderOp<H>;
    type O2 = NullOp;

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

        let mut tx = [0x00u8; 2 * (size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>())];

        let mut op = PulseWidthEncoderOp::new(|i| PulseWidth::new(i.0).unwrap());

        assert_eq!(
            size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>(),
            op.required_size(&device)
        );

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::ConfigPulseWidthEncoderV10 as u8);
        assert!((0..PWE_BUF_SIZE).all(|i| i as u8 == tx[2 + i]));
    }
}
