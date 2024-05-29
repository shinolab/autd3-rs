use crate::firmware::{
    fpga::{DebugType, GPIOOut},
    operation::DebugSettingOp,
};

use crate::datagram::*;

pub struct DebugSettings<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> {
    f: F,
}

impl<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> DebugSettings<F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct DebugSettingOpGenerator<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> {
    f: F,
}

impl<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> OperationGenerator
    for DebugSettingOpGenerator<F>
{
    type O1 = DebugSettingOp;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                [GPIOOut::O0, GPIOOut::O1, GPIOOut::O2, GPIOOut::O3]
                    .map(|gpio| (self.f)(device, gpio).ty()),
                [GPIOOut::O0, GPIOOut::O1, GPIOOut::O2, GPIOOut::O3]
                    .map(|gpio| (self.f)(device, gpio).value()),
            ),
            Self::O2::default(),
        )
    }
}

impl<'a, F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> Datagram<'a> for DebugSettings<F> {
    type O1 = DebugSettingOp;
    type O2 = NullOp;
    type G = DebugSettingOpGenerator<F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(DebugSettingOpGenerator { f: self.f })
    }
}
