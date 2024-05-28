use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{DebugType, GPIOOut},
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

#[repr(C, align(2))]
struct DebugSetting {
    tag: TypeTag,
    __pad: u8,
    ty: [u8; 4],
    value: [u16; 4],
}

pub struct DebugSettingOp<'a, F: Fn(GPIOOut) -> DebugType<'a>> {
    is_done: bool,
    f: F,
}

impl<'a, F: Fn(GPIOOut) -> DebugType<'a>> DebugSettingOp<'a, F> {
    pub fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<'a, F: Fn(GPIOOut) -> DebugType<'a> + Send + Sync> Operation for DebugSettingOp<'a, F> {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<DebugSetting>(tx) = DebugSetting {
            tag: TypeTag::Debug,
            __pad: 0,
            ty: [GPIOOut::O0, GPIOOut::O1, GPIOOut::O2, GPIOOut::O3]
                .map(|gpio| (self.f)(gpio).ty()),
            value: [GPIOOut::O0, GPIOOut::O1, GPIOOut::O2, GPIOOut::O3]
                .map(|gpio| (self.f)(gpio).value()),
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
