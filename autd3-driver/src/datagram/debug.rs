use crate::firmware::{
    fpga::{DebugType, GPIOOut},
    operation::DebugSettingOp,
};

use crate::datagram::*;
use derive_more::Debug;
use derive_new::new;

#[derive(Debug, new)]
pub struct DebugSettings<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> {
    #[debug(ignore)]
    f: F,
}

pub struct DebugSettingOpGenerator<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> {
    f: F,
}

impl<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> OperationGenerator
    for DebugSettingOpGenerator<F>
{
    type O1 = DebugSettingOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                [GPIOOut::O0, GPIOOut::O1, GPIOOut::O2, GPIOOut::O3]
                    .map(|gpio| (self.f)(device, gpio).into()),
            ),
            Self::O2::new(),
        )
    }
}

impl<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> Datagram for DebugSettings<F> {
    type G = DebugSettingOpGenerator<F>;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(DebugSettingOpGenerator { f: self.f })
    }
}
