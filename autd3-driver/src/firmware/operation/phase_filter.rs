use crate::{
    derive::Transducer,
    error::AUTDInternalError,
    firmware::{
        fpga::Phase,
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

#[repr(C, align(2))]
struct PhaseFilter {
    tag: TypeTag,
}

pub struct PhaseFilterOp<P: Into<Phase>, F: Fn(&Transducer) -> P + Send + Sync> {
    is_done: bool,
    f: F,
}

impl<P: Into<Phase>, F: Fn(&Transducer) -> P + Send + Sync> PhaseFilterOp<P, F> {
    pub fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<P: Into<Phase>, F: Fn(&Transducer) -> P + Send + Sync> Operation for PhaseFilterOp<P, F> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        cast::<PhaseFilter>(tx).tag = TypeTag::PhaseFilter;

        unsafe {
            std::slice::from_raw_parts_mut(
                tx[std::mem::size_of::<PhaseFilter>()..].as_mut_ptr() as *mut Phase,
                device.num_transducers(),
            )
            .iter_mut()
            .zip(device.iter())
            .for_each(|(d, s)| *d = (self.f)(s).into());
        }

        self.is_done = true;
        Ok(std::mem::size_of::<PhaseFilter>()
            + (((device.num_transducers() + 1) >> 1) << 1) * std::mem::size_of::<Phase>())
    }

    fn required_size(&self, device: &Device) -> usize {
        std::mem::size_of::<PhaseFilter>()
            + (((device.num_transducers() + 1) >> 1) << 1) * std::mem::size_of::<Phase>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8;
            (std::mem::size_of::<PhaseFilter>()
                + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>())];

        let mut op = PhaseFilterOp::new(|tr| Phase::new(tr.idx() as u8));

        assert_eq!(
            std::mem::size_of::<PhaseFilter>()
                + (NUM_TRANS_IN_UNIT + 1) / 2 * 2 * std::mem::size_of::<Phase>(),
            op.required_size(&device)
        );

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::PhaseFilter as u8);
        (0..device.num_transducers()).for_each(|i| {
            assert_eq!(tx[std::mem::size_of::<PhaseFilter>() + i], i as u8);
        });
    }
}
