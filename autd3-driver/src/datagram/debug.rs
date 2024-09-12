use crate::firmware::{
    fpga::{DebugType, GPIOOut},
    operation::DebugSettingOp,
};

use crate::datagram::*;
use derive_more::Debug;

#[derive(Debug)]
pub struct DebugSettings<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> {
    #[debug(ignore)]
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
                    .map(|gpio| (self.f)(device, gpio).into()),
            ),
            Self::O2::default(),
        )
    }
}

impl<F: Fn(&Device, GPIOOut) -> DebugType + Send + Sync> Datagram for DebugSettings<F> {
    type G = DebugSettingOpGenerator<F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(DebugSettingOpGenerator { f: self.f })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}
