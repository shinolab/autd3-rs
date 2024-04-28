use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::DebugType,
        operation::{cast, Operation, TypeTag},
    },
    geometry::{Device, Geometry},
};

use super::Remains;

#[repr(C, align(2))]
struct DebugSetting {
    tag: TypeTag,
    __pad: u8,
    ty: [u8; 4],
    value: [u16; 4],
}

pub struct DebugSettingOp<F: Fn(&Device) -> [DebugType; 4]> {
    remains: Remains,
    f: F,
}

impl<F: Fn(&Device) -> [DebugType; 4]> DebugSettingOp<F> {
    pub fn new(f: F) -> Self {
        Self {
            remains: Default::default(),
            f,
        }
    }
}

impl<F: Fn(&Device) -> [DebugType; 4]> Operation for DebugSettingOp<F> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<DebugSetting>(tx) = DebugSetting {
            tag: TypeTag::Debug,
            __pad: 0,
            ty: (self.f)(device).map(|t| t.ty()),
            value: (self.f)(device).map(|t| t.value()),
        };

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<DebugSetting>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<DebugSetting>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, 1);
        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}
