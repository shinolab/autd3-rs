use std::mem::size_of;

use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::Phase,
        operation::{write_to_tx, Operation, TypeTag},
    },
    geometry::{Device, Transducer},
};

#[repr(C, align(2))]
struct PhaseCorr {
    tag: TypeTag,
}

pub struct PhaseCorrectionOp<F: Fn(&Transducer) -> Phase> {
    is_done: bool,
    f: F,
}

impl<F: Fn(&Transducer) -> Phase> PhaseCorrectionOp<F> {
    pub const fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<F: Fn(&Transducer) -> Phase + Send + Sync> Operation for PhaseCorrectionOp<F> {
    fn pack(&mut self, dev: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        unsafe {
            write_to_tx(
                PhaseCorr {
                    tag: TypeTag::PhaseCorrection,
                },
                tx,
            );
        }

        tx[size_of::<PhaseCorr>()..]
            .iter_mut()
            .zip(dev.iter())
            .for_each(|(x, tr)| {
                *x = (self.f)(tr).value();
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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 2 * (size_of::<PhaseCorr>() + 250)];

        let mut op = PhaseCorrectionOp::new(|tr| Phase::new(tr.idx() as _));

        assert_eq!(size_of::<PhaseCorr>() + 250, op.required_size(&device));

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::PhaseCorrection as u8);
        assert!((0..249).all(|i| i as u8 == tx[size_of::<PhaseCorr>() + i]));
    }
}
