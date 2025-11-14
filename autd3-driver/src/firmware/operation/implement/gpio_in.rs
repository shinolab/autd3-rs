use std::{convert::Infallible, mem::size_of};

use crate::{
    datagram::EmulateGPIOIn,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::{firmware::GPIOIn, geometry::Device};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GPIOInFlags(u8);

impl GPIOInFlags {
    const NONE: GPIOInFlags = GPIOInFlags(0);
    const GPIO_IN_0: u8 = 1 << 0;
    const GPIO_IN_1: u8 = 1 << 1;
    const GPIO_IN_2: u8 = 1 << 2;
    const GPIO_IN_3: u8 = 1 << 3;

    fn set(&mut self, bit: u8, value: bool) {
        if value {
            self.0 |= bit;
        }
    }
}

#[repr(C, align(2))]
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
        crate::firmware::operation::write_to_tx(
            tx,
            EmulateGPIOInMsg {
                tag: TypeTag::EmulateGPIOIn,
                flag: {
                    let mut flag = GPIOInFlags::NONE;
                    flag.set(GPIOInFlags::GPIO_IN_0, self.value[0]);
                    flag.set(GPIOInFlags::GPIO_IN_1, self.value[1]);
                    flag.set(GPIOInFlags::GPIO_IN_2, self.value[2]);
                    flag.set(GPIOInFlags::GPIO_IN_3, self.value[3]);
                    flag
                },
            },
        );

        self.is_done = true;
        Ok(size_of::<EmulateGPIOInMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<EmulateGPIOInMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<'a, H: Fn(GPIOIn) -> bool, F: Fn(&'a Device) -> H> OperationGenerator<'a>
    for EmulateGPIOIn<F, H>
{
    type O1 = EmulateGPIOInOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new([GPIOIn::I0, GPIOIn::I1, GPIOIn::I2, GPIOIn::I3].map((self.f)(device))),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use super::*;

    #[rstest::rstest]
    #[case(0b1001, [true, false, false, true])]
    #[case(0b0110, [false, true, true, false])]
    fn op(#[case] expected: u8, #[case] value: [bool; 4]) {
        let device = crate::tests::create_device();

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
