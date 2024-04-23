use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    fpga::DebugType,
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

#[repr(C, align(2))]
struct DebugSetting {
    tag: TypeTag,
    __pad: u8,
    ty: [u8; 4],
    value: [u16; 4],
}

pub struct DebugSettingOp<F: Fn(&Device) -> [DebugType; 4]> {
    remains: HashMap<usize, usize>,
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
        assert_eq!(self.remains[&device.idx()], 1);

        *cast::<DebugSetting>(tx) = DebugSetting {
            tag: TypeTag::Debug,
            __pad: 0,
            ty: (self.f)(device).map(|t| t.ty()),
            value: (self.f)(device).map(|t| t.value()),
        };

        Ok(std::mem::size_of::<DebugSetting>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<DebugSetting>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains = geometry.devices().map(|device| (device.idx(), 1)).collect();
        Ok(())
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }

    fn commit(&mut self, device: &Device) {
        self.remains.insert(device.idx(), 0);
    }
}
