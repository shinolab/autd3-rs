use std::mem::size_of;

use crate::{
    error::AUTDInternalError,
    firmware::operation::{write_to_tx, Operation, TypeTag},
    geometry::Device,
};

#[repr(C, align(2))]
struct CpuGPIOOut {
    tag: TypeTag,
    pa_podr: u8,
}

pub struct CpuGPIOOutOp {
    is_done: bool,
    pa5: bool,
    pa7: bool,
}

impl CpuGPIOOutOp {
    pub const fn new(pa5: bool, pa7: bool) -> Self {
        Self {
            is_done: false,
            pa5,
            pa7,
        }
    }
}

impl Operation for CpuGPIOOutOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        write_to_tx(
            CpuGPIOOut {
                tag: TypeTag::CpuGPIOOut,
                pa_podr: ((self.pa5 as u8) << 5) | ((self.pa7 as u8) << 7),
            },
            tx,
        );

        self.is_done = true;
        Ok(size_of::<CpuGPIOOut>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<CpuGPIOOut>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::tests::create_device;

    use super::*;

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[rstest::rstest]
    #[test]
    #[case(0b10100000, true, true)]
    #[case(0b00100000, true, false)]
    #[case(0b10000000, false, true)]
    #[case(0b00000000, false, false)]
    fn debug_op(#[case] expect: u8, #[case] pa5: bool, #[case] pa7: bool) {
        const FRAME_SIZE: usize = size_of::<CpuGPIOOut>();

        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op = CpuGPIOOutOp::new(pa5, pa7);

        assert_eq!(size_of::<CpuGPIOOut>(), op.required_size(&device));
        assert_eq!(Ok(size_of::<CpuGPIOOut>()), op.pack(&device, &mut tx));
        assert!(op.is_done());
        assert_eq!(TypeTag::CpuGPIOOut as u8, tx[0]);
        assert_eq!(expect, tx[1]);
    }
}