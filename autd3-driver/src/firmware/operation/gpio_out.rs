use std::{convert::Infallible, mem::size_of};

use crate::{
    firmware::{
        fpga::GPIOOutValue,
        operation::{Operation, TypeTag},
    },
    geometry::Device,
};

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct GPIOOutput {
    tag: TypeTag,
    __: [u8; 7],
    value: [GPIOOutValue; 4],
}

pub struct GPIOOutputOp {
    is_done: bool,
    value: [GPIOOutValue; 4],
}

impl GPIOOutputOp {
    pub(crate) const fn new(value: [GPIOOutValue; 4]) -> Self {
        Self {
            is_done: false,
            value,
        }
    }
}

impl Operation for GPIOOutputOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        super::write_to_tx(
            tx,
            GPIOOutput {
                tag: TypeTag::Debug,
                __: [0; 7],
                value: self.value,
            },
        );

        self.is_done = true;
        Ok(size_of::<GPIOOutput>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<GPIOOutput>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::operation::tests::create_device;

    use super::*;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn gpio_output_op() {
        const FRAME_SIZE: usize = size_of::<GPIOOutput>();

        let device = create_device(NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op = GPIOOutputOp::new([
            GPIOOutValue::new()
                .with_tag(0x01)
                .with_value(0x02030405060708),
            GPIOOutValue::new()
                .with_tag(0x11)
                .with_value(0x12131415161718),
            GPIOOutValue::new()
                .with_tag(0x10)
                .with_value(0x20304050607080),
            GPIOOutValue::new()
                .with_tag(0x11)
                .with_value(0x21314151617181),
        ]);

        assert_eq!(size_of::<GPIOOutput>(), op.required_size(&device));
        assert_eq!(Ok(size_of::<GPIOOutput>()), op.pack(&device, &mut tx));
        assert!(op.is_done());
        assert_eq!(TypeTag::Debug as u8, tx[0]);
        assert_eq!(0x08, tx[8]);
        assert_eq!(0x07, tx[9]);
        assert_eq!(0x06, tx[10]);
        assert_eq!(0x05, tx[11]);
        assert_eq!(0x04, tx[12]);
        assert_eq!(0x03, tx[13]);
        assert_eq!(0x02, tx[14]);
        assert_eq!(0x01, tx[15]);
        assert_eq!(0x18, tx[16]);
        assert_eq!(0x17, tx[17]);
        assert_eq!(0x16, tx[18]);
        assert_eq!(0x15, tx[19]);
        assert_eq!(0x14, tx[20]);
        assert_eq!(0x13, tx[21]);
        assert_eq!(0x12, tx[22]);
        assert_eq!(0x11, tx[23]);
        assert_eq!(0x80, tx[24]);
        assert_eq!(0x70, tx[25]);
        assert_eq!(0x60, tx[26]);
        assert_eq!(0x50, tx[27]);
        assert_eq!(0x40, tx[28]);
        assert_eq!(0x30, tx[29]);
        assert_eq!(0x20, tx[30]);
        assert_eq!(0x10, tx[31]);
        assert_eq!(0x81, tx[32]);
        assert_eq!(0x71, tx[33]);
        assert_eq!(0x61, tx[34]);
        assert_eq!(0x51, tx[35]);
        assert_eq!(0x41, tx[36]);
        assert_eq!(0x31, tx[37]);
        assert_eq!(0x21, tx[38]);
        assert_eq!(0x11, tx[39]);
    }
}
