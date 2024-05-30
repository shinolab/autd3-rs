use std::mem::size_of;

use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
    geometry::Device,
};

#[repr(C, align(2))]
struct DebugSetting {
    tag: TypeTag,
    __pad: u8,
    ty: [u8; 4],
    value: [u16; 4],
}

pub struct DebugSettingOp {
    is_done: bool,
    ty: [u8; 4],
    value: [u16; 4],
}

impl DebugSettingOp {
    pub fn new(ty: [u8; 4], value: [u16; 4]) -> Self {
        Self {
            is_done: false,
            ty,
            value,
        }
    }
}

impl Operation for DebugSettingOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<DebugSetting>(tx) = DebugSetting {
            tag: TypeTag::Debug,
            __pad: 0,
            ty: self.ty,
            value: self.value,
        };

        self.is_done = true;
        Ok(size_of::<DebugSetting>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<DebugSetting>()
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

    #[test]
    fn debug_op() {
        const FRAME_SIZE: usize = size_of::<DebugSetting>();

        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op =
            DebugSettingOp::new([0x01, 0x02, 0x03, 0x04], [0x0605, 0x0807, 0x0a09, 0x0c0b]);

        assert_eq!(size_of::<DebugSetting>(), op.required_size(&device));
        assert_eq!(Ok(size_of::<DebugSetting>()), op.pack(&device, &mut tx));
        assert_eq!(op.is_done(), true);
        assert_eq!(TypeTag::Debug as u8, tx[0]);
        assert_eq!(0x01, tx[2]);
        assert_eq!(0x02, tx[3]);
        assert_eq!(0x03, tx[4]);
        assert_eq!(0x04, tx[5]);
        assert_eq!(0x05, tx[6]);
        assert_eq!(0x06, tx[7]);
        assert_eq!(0x07, tx[8]);
        assert_eq!(0x08, tx[9]);
        assert_eq!(0x09, tx[10]);
        assert_eq!(0x0a, tx[11]);
        assert_eq!(0x0b, tx[12]);
        assert_eq!(0x0c, tx[13]);
    }
}
