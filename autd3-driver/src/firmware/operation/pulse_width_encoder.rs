use std::{convert::Infallible, mem::size_of};

use crate::{
    firmware::{
        fpga::PWE_BUF_SIZE,
        operation::{Operation, TypeTag},
    },
    geometry::Device,
};

use autd3_core::gain::EmitIntensity;
use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct Pwe {
    tag: TypeTag,
    __: u8,
}

pub struct PulseWidthEncoderOp<F: Fn(EmitIntensity) -> u8> {
    is_done: bool,
    f: F,
}

impl<F: Fn(EmitIntensity) -> u8> PulseWidthEncoderOp<F> {
    pub(crate) const fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<F: Fn(EmitIntensity) -> u8 + Send + Sync> Operation for PulseWidthEncoderOp<F> {
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
            .iter_mut()
            .take(PWE_BUF_SIZE)
            .enumerate()
            .for_each(|(i, x)| {
                *x = (self.f)(EmitIntensity(i as u8));
            });

        self.is_done = true;

        Ok(size_of::<Pwe>() + PWE_BUF_SIZE)
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<Pwe>() + PWE_BUF_SIZE
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

        let mut tx = [0x00u8; 2 * (size_of::<Pwe>() + PWE_BUF_SIZE)];

        let mut op = PulseWidthEncoderOp::new(|i| i.0);

        assert_eq!(size_of::<Pwe>() + PWE_BUF_SIZE, op.required_size(&device));

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::ConfigPulseWidthEncoder as u8);
        assert!((0..PWE_BUF_SIZE).all(|i| i as u8 == tx[2 + i]));
    }
}
