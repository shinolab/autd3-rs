use std::{convert::Infallible, mem::size_of};

use super::OperationGenerator;
use crate::{
    datagram::CpuGPIOOutputs,
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
};

use autd3_core::{firmware::CpuGPIOPort, geometry::Device};

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct CpuGPIOOutMsg {
    tag: TypeTag,
    pa_podr: u8,
}

pub struct CpuGPIOOutputsOp {
    is_done: bool,
    pa5: bool,
    pa7: bool,
}

impl CpuGPIOOutputsOp {
    pub(crate) const fn new(pa5: bool, pa7: bool) -> Self {
        Self {
            is_done: false,
            pa5,
            pa7,
        }
    }
}

impl Operation<'_> for CpuGPIOOutputsOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
            tx,
            CpuGPIOOutMsg {
                tag: TypeTag::CpuGPIOOut,
                pa_podr: ((self.pa5 as u8) << 5) | ((self.pa7 as u8) << 7),
            },
        );

        self.is_done = true;
        Ok(size_of::<CpuGPIOOutMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<CpuGPIOOutMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<F: Fn(&Device) -> CpuGPIOPort + Send + Sync> OperationGenerator<'_> for CpuGPIOOutputs<F> {
    type O1 = CpuGPIOOutputsOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        let port = (self.f)(device);
        Some((CpuGPIOOutputsOp::new(port.pa5, port.pa7), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(0b10100000, true, true)]
    #[case(0b00100000, true, false)]
    #[case(0b10000000, false, true)]
    #[case(0b00000000, false, false)]
    fn cpu_gpio_out_op(#[case] expect: u8, #[case] pa5: bool, #[case] pa7: bool) {
        const FRAME_SIZE: usize = size_of::<CpuGPIOOutMsg>();

        let device = crate::autd3_device::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op = CpuGPIOOutputsOp::new(pa5, pa7);

        assert_eq!(size_of::<CpuGPIOOutMsg>(), op.required_size(&device));
        assert_eq!(Ok(size_of::<CpuGPIOOutMsg>()), op.pack(&device, &mut tx));
        assert!(op.is_done());
        assert_eq!(TypeTag::CpuGPIOOut as u8, tx[0]);
        assert_eq!(expect, tx[1]);
    }
}
