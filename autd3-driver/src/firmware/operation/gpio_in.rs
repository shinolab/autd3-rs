use std::convert::Infallible;

use crate::{
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

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
struct EmulateGPIOIn {
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

impl Operation for EmulateGPIOInOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        let mut flag = GPIOInFlags::NONE;
        seq_macro::seq!(N in 0..4 {#(flag.set(GPIOInFlags::GPIO_IN_~N, self.value[N]);)*});

        super::write_to_tx(
            tx,
            EmulateGPIOIn {
                tag: TypeTag::EmulateGPIOIn,
                flag,
            },
        );

        self.is_done = true;
        Ok(std::mem::size_of::<EmulateGPIOIn>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<EmulateGPIOIn>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use super::*;
    use crate::firmware::operation::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[rstest::rstest]
    #[test]
    #[case(0b1001, [true, false, false, true])]
    #[case(0b0110, [false, true, true, false])]
    fn test(#[case] expected: u8, #[case] value: [bool; 4]) {
        let device = create_device(NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<EmulateGPIOIn>()];

        let mut op = EmulateGPIOInOp::new(value);

        assert_eq!(op.required_size(&device), size_of::<EmulateGPIOIn>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::EmulateGPIOIn as u8);
        assert_eq!(tx[offset_of!(EmulateGPIOIn, flag)], expected);
    }
}
