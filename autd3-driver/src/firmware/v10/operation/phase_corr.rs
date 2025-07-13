use std::{convert::Infallible, mem::size_of};

use super::OperationGenerator;
use crate::{
    datagram::PhaseCorrection,
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
};

use autd3_core::{
    gain::Phase,
    geometry::{Device, Transducer},
};

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct PhaseCorr {
    tag: TypeTag,
    __: u8,
}

pub struct PhaseCorrectionOp<F> {
    is_done: bool,
    f: F,
}

impl<'tr, F: Fn(&'tr Transducer) -> Phase> PhaseCorrectionOp<F> {
    pub(crate) const fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<'dev, 'tr, F: Fn(&'tr Transducer) -> Phase + Send + Sync> Operation<'dev>
    for PhaseCorrectionOp<F>
where
    'dev: 'tr,
{
    type Error = Infallible;

    fn pack(&mut self, dev: &'dev Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
            tx,
            PhaseCorr {
                tag: TypeTag::PhaseCorrection,
                __: 0,
            },
        );

        tx[size_of::<PhaseCorr>()..]
            .chunks_mut(size_of::<Phase>())
            .zip(dev.iter())
            .for_each(|(dst, tr)| {
                crate::firmware::driver::write_to_tx(dst, (self.f)(tr));
            });

        self.is_done = true;

        Ok(size_of::<PhaseCorr>() + ((dev.num_transducers() + 1) & !0x1))
    }

    fn required_size(&self, dev: &Device) -> usize {
        size_of::<PhaseCorr>() + ((dev.num_transducers() + 1) & !0x1)
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<'dev, 'tr, FT: Fn(&'tr Transducer) -> Phase + Send + Sync, F: Fn(&'dev Device) -> FT>
    OperationGenerator<'dev> for PhaseCorrection<F, FT>
where
    'dev: 'tr,
{
    type O1 = PhaseCorrectionOp<FT>;
    type O2 = NullOp;

    fn generate(&mut self, device: &'dev Device) -> Option<(Self::O1, Self::O2)> {
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

        let mut tx = [0x00u8; 2 * (size_of::<PhaseCorr>() + 250)];

        let mut op = PhaseCorrectionOp::new(|tr| Phase(tr.idx() as _));

        assert_eq!(size_of::<PhaseCorr>() + 250, op.required_size(&device));

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::PhaseCorrection as u8);
        assert!((0..249).all(|i| i as u8 == tx[size_of::<PhaseCorr>() + i]));
    }
}
