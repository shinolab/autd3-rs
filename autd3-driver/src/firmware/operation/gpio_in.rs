use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::GPIOIn,
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

#[derive(Clone, Copy)]
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
struct EmulateGPIOIn {
    tag: TypeTag,
    flag: GPIOInFlags,
}

pub struct EmulateGPIOInOp<F: Fn(GPIOIn) -> bool> {
    is_done: bool,
    f: F,
}

impl<F: Fn(GPIOIn) -> bool> EmulateGPIOInOp<F> {
    pub fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<F: Fn(GPIOIn) -> bool> Operation for EmulateGPIOInOp<F> {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let mut flag = GPIOInFlags::NONE;
        flag.set(GPIOInFlags::GPIO_IN_0, (self.f)(GPIOIn::I0));
        flag.set(GPIOInFlags::GPIO_IN_1, (self.f)(GPIOIn::I1));
        flag.set(GPIOInFlags::GPIO_IN_2, (self.f)(GPIOIn::I2));
        flag.set(GPIOInFlags::GPIO_IN_3, (self.f)(GPIOIn::I3));

        *cast::<EmulateGPIOIn>(tx) = EmulateGPIOIn {
            tag: TypeTag::EmulateGPIOIn,
            flag,
        };

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
    use crate::geometry::tests::create_device;

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[rstest::rstest]
    #[test]
    #[case(0b1001, |gpio| gpio == GPIOIn::I0 || gpio == GPIOIn::I3)]
    #[case(0b0110, |gpio| gpio == GPIOIn::I1 || gpio == GPIOIn::I2)]
    fn test(#[case] expected: u8, #[case] f: impl Fn(GPIOIn) -> bool) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<EmulateGPIOIn>()];

        let mut op = EmulateGPIOInOp::new(f);

        assert_eq!(op.required_size(&device), size_of::<EmulateGPIOIn>());

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::EmulateGPIOIn as u8);
        assert_eq!(tx[offset_of!(EmulateGPIOIn, flag)], expected);
    }
}
