use std::convert::Infallible;

use super::OperationGenerator;
use crate::{
    datagram::EmulateGPIOIn,
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
};

use autd3_core::{firmware::GPIOIn, geometry::Device};

use zerocopy::{Immutable, IntoBytes};

#[derive(Clone, Copy, IntoBytes, Immutable)]
#[repr(C)]
pub struct GPIOInFlags(u8);

bitflags::bitflags! {
    impl GPIOInFlags : u8 {
        const NONE      = 0;
        const GPIO_IN_0 = 1 << 0;
        const GPIO_IN_1 = 1 << 1;
        const GPIO_IN_2 = 1 << 2;
        const GPIO_IN_3 = 1 << 3;
    }
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct EmulateGPIOInMsg {
    tag: TypeTag,
    flag: GPIOInFlags,
}

pub struct EmulateGPIOInOp {
    is_done: bool,
    value: [bool; 4],
}

impl EmulateGPIOInOp {
    pub(crate) const fn new(value: [bool; 4]) -> Self {
        Self {
            is_done: false,
            value,
        }
    }
}

impl Operation<'_> for EmulateGPIOInOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        let mut flag = GPIOInFlags::NONE;
        seq_macro::seq!(N in 0..4 {#(flag.set(GPIOInFlags::GPIO_IN_~N, self.value[N]);)*});

        crate::firmware::driver::write_to_tx(
            tx,
            EmulateGPIOInMsg {
                tag: TypeTag::EmulateGPIOIn,
                flag,
            },
        );

        self.is_done = true;
        Ok(std::mem::size_of::<EmulateGPIOInMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<EmulateGPIOInMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<H: Fn(GPIOIn) -> bool + Send + Sync, F: Fn(&Device) -> H> OperationGenerator<'_>
    for EmulateGPIOIn<F>
{
    type O1 = EmulateGPIOInOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        let h = (self.f)(device);
        Some((
            Self::O1::new([h(GPIOIn::I0), h(GPIOIn::I1), h(GPIOIn::I2), h(GPIOIn::I3)]),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use super::*;

    #[rstest::rstest]
    #[case(0b1001, [true, false, false, true])]
    #[case(0b0110, [false, true, true, false])]
    fn test(#[case] expected: u8, #[case] value: [bool; 4]) {
        let device = crate::autd3_device::tests::create_device();

        let mut tx = [0x00u8; size_of::<EmulateGPIOInMsg>()];

        let mut op = EmulateGPIOInOp::new(value);

        assert_eq!(op.required_size(&device), size_of::<EmulateGPIOInMsg>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::EmulateGPIOIn as u8);
        assert_eq!(tx[offset_of!(EmulateGPIOInMsg, flag)], expected);
    }
}
