use std::{convert::Infallible, mem::size_of};

use super::OperationGenerator;
use crate::{
    datagram::OutputMask,
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
    geometry::{Device, Transducer},
};

use autd3_core::datagram::Segment;

use itertools::Itertools;
use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct OutputMaskT {
    tag: TypeTag,
    segment: u8,
}

pub struct OutputMaskOp<F> {
    is_done: bool,
    f: F,
    segment: Segment,
}

impl<F: Fn(&Transducer) -> bool> OutputMaskOp<F> {
    pub(crate) const fn new(f: F, segment: Segment) -> Self {
        Self {
            is_done: false,
            f,
            segment,
        }
    }
}

impl<F: Fn(&Transducer) -> bool + Send + Sync> Operation for OutputMaskOp<F> {
    type Error = Infallible;

    fn pack(&mut self, dev: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
            tx,
            OutputMaskT {
                tag: TypeTag::OutputMask,
                segment: self.segment as u8,
            },
        );

        tx[size_of::<OutputMaskT>()..]
            .iter_mut()
            .zip(&dev.iter().chunks(8))
            .for_each(|(dst, chunk)| {
                *dst =
                    chunk.enumerate().fold(
                        0x00u8,
                        |acc, (i, tr)| {
                            if (self.f)(tr) { acc | (1 << i) } else { acc }
                        },
                    );
            });

        self.is_done = true;

        Ok(size_of::<OutputMaskT>() + dev.num_transducers().div_ceil(8))
    }

    fn required_size(&self, dev: &Device) -> usize {
        size_of::<OutputMaskT>() + dev.num_transducers().div_ceil(8)
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<FT: Fn(&Transducer) -> bool + Send + Sync, F: Fn(&Device) -> FT> OperationGenerator
    for OutputMask<F>
{
    type O1 = OutputMaskOp<FT>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new((self.f)(device), self.segment), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[rstest::rstest]
    #[case(0xFF, true)]
    #[case(0x00, false)]
    #[test]
    fn test(#[case] expected: u8, #[case] f: bool) {
        let device = crate::autd3_device::tests::create_device();

        let mut tx = [0x00u8; 2 * (size_of::<OutputMaskT>() + 32)];

        let mut op = OutputMaskOp::new(|_| f, Segment::S0);

        assert_eq!(size_of::<OutputMaskT>() + 32, op.required_size(&device));

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::OutputMask as u8);
        assert_eq!(tx[1], Segment::S0 as u8);
        (0..31).for_each(|i| assert_eq!(expected, tx[size_of::<OutputMaskT>() + i]));
        assert_eq!(expected & 0x01, tx[size_of::<OutputMaskT>() + 31]);
    }
}
