use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    geometry::{Device, Geometry, Transducer},
    operation::{cast, Operation, TypeTag},
};

#[repr(C, align(2))]
struct DebugSetting {
    tag: TypeTag,
    __pad: u8,
    ty: [u8; 4],
    value: [u16; 4],
}

#[non_exhaustive]
pub enum DebugType<'a> {
    None,
    BaseSignal,
    PwmOut(&'a Transducer),
}

impl From<&DebugType<'_>> for u8 {
    fn from(ty: &DebugType) -> u8 {
        match ty {
            DebugType::None => 0,
            DebugType::BaseSignal => 1,
            DebugType::PwmOut(_) => 2,
        }
    }
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

        let d = cast::<DebugSetting>(tx);
        d.tag = TypeTag::Debug;
        let types = (self.f)(device);
        for (i, ty) in types.iter().enumerate() {
            d.ty[i] = ty.into();
            d.value[i] = match ty {
                DebugType::None | DebugType::BaseSignal => 0,
                DebugType::PwmOut(tr) => tr.idx() as _,
            }
        }

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

mod old {
    #![allow(deprecated)]
    use super::*;

    #[deprecated(note = "Use DebugSettingOp instead", since = "22.1.0")]
    pub struct DebugOutIdxOp<F: Fn(&Device) -> Option<&Transducer>> {
        remains: HashMap<usize, usize>,
        f: F,
    }

    impl<F: Fn(&Device) -> Option<&Transducer>> DebugOutIdxOp<F> {
        pub fn new(f: F) -> Self {
            Self {
                remains: Default::default(),
                f,
            }
        }
    }

    impl<F: Fn(&Device) -> Option<&Transducer>> Operation for DebugOutIdxOp<F> {
        fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
            assert_eq!(self.remains[&device.idx()], 1);

            let d = cast::<DebugSetting>(tx);
            d.tag = TypeTag::Debug;

            d.ty[0] = (&DebugType::BaseSignal).into();
            d.value[0] = 0;
            d.ty[1] = if let Some(tr) = (self.f)(device) {
                (&DebugType::PwmOut(tr)).into()
            } else {
                (&DebugType::None).into()
            };
            d.value[1] = (self.f)(device).map(|tr| tr.idx() as u16).unwrap_or(0);
            d.ty[2] = (&DebugType::None).into();
            d.value[2] = 0;
            d.ty[3] = (&DebugType::None).into();
            d.value[3] = 0;

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
}

#[allow(deprecated)]
pub use old::DebugOutIdxOp;
