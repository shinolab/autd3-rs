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
        Ok(std::mem::size_of::<DebugSetting>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<DebugSetting>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}
