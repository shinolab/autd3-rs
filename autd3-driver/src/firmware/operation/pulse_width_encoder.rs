use std::{convert::Infallible, mem::size_of};

use crate::{
    firmware::{
        fpga::{PWE_BUF_SIZE, PulseWidth},
        operation::{Operation, TypeTag},
    },
    geometry::Device,
};

use autd3_core::{common::ULTRASOUND_PERIOD_COUNT_BITS, gain::EmitIntensity};
use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct Pwe {
    tag: TypeTag,
    __: u8,
}

pub struct PulseWidthEncoderOp<
    F: Fn(EmitIntensity) -> PulseWidth<u16, ULTRASOUND_PERIOD_COUNT_BITS>,
> {
    is_done: bool,
    f: F,
}

impl<F: Fn(EmitIntensity) -> PulseWidth<u16, ULTRASOUND_PERIOD_COUNT_BITS>> PulseWidthEncoderOp<F> {
    pub(crate) const fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<F: Fn(EmitIntensity) -> PulseWidth<u16, ULTRASOUND_PERIOD_COUNT_BITS> + Send + Sync> Operation
    for PulseWidthEncoderOp<F>
{
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        super::write_to_tx(
            tx,
            Pwe {
                tag: TypeTag::ConfigPulseWidthEncoder,
                __: 0,
            },
        );

        tx[size_of::<Pwe>()..]
            .chunks_mut(size_of::<u16>())
            .take(PWE_BUF_SIZE)
            .enumerate()
            .for_each(|(i, dst)| {
                super::write_to_tx(dst, (self.f)(EmitIntensity(i as u8)).pulse_width());
            });

        self.is_done = true;

        Ok(size_of::<Pwe>() + PWE_BUF_SIZE * size_of::<u16>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<Pwe>() + PWE_BUF_SIZE * size_of::<u16>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::firmware::operation::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn test() {
        let device = create_device(NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 2 * (size_of::<Pwe>() + PWE_BUF_SIZE * size_of::<u16>())];

        let mut op = PulseWidthEncoderOp::new(|i| PulseWidth::new(i.0 as u16).unwrap());

        assert_eq!(
            size_of::<Pwe>() + PWE_BUF_SIZE * size_of::<u16>(),
            op.required_size(&device)
        );

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::ConfigPulseWidthEncoder as u8);
        assert!((0..PWE_BUF_SIZE).all(|i| i as u8 == tx[2 + 2 * i]));
    }
}
