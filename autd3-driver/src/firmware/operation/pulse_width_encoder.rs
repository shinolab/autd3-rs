use std::mem::size_of;

use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::PWE_BUF_SIZE,
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

#[repr(C, align(2))]
struct Pwe {
    tag: TypeTag,
}

pub struct PulseWidthEncoderOp<F: Fn(u8) -> u8> {
    is_done: bool,
    f: F,
}

impl<F: Fn(u8) -> u8> PulseWidthEncoderOp<F> {
    pub const fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<F: Fn(u8) -> u8 + Send + Sync> Operation for PulseWidthEncoderOp<F> {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<Pwe>(tx) = Pwe {
            tag: TypeTag::ConfigPulseWidthEncoder,
        };

        tx[size_of::<Pwe>()..]
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| {
                *x = (self.f)(i as u8);
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
