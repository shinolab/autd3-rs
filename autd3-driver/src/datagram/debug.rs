use crate::firmware::{
    fpga::{DebugType, GPIOOut},
    operation::DebugSettingOp,
};

use crate::datagram::*;

pub struct DebugSettings<'a, H: Fn(GPIOOut) -> DebugType<'a>, F: Fn(&Device) -> H + Send + Sync> {
    f: F,
    _phantom: std::marker::PhantomData<&'a H>,
}

impl<'a, H: Fn(GPIOOut) -> DebugType<'a>, F: Fn(&Device) -> H + Send + Sync>
    DebugSettings<'a, H, F>
{
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct DebugSettingOpGenerator<
    'a,
    H: Fn(GPIOOut) -> DebugType<'a> + Send + Sync + 'a,
    F: Fn(&Device) -> H + Send + Sync + 'a,
> {
    f: F,
}

impl<'a, H: Fn(GPIOOut) -> DebugType<'a> + Send + Sync + 'a, F: Fn(&Device) -> H + Send + Sync>
    OperationGenerator for DebugSettingOpGenerator<'a, H, F>
{
    type O1 = DebugSettingOp<'a, H>;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new((self.f)(device)), Self::O2::default()))
    }
}

impl<
        'a,
        H: Fn(GPIOOut) -> DebugType<'a> + Send + Sync + 'a,
        F: Fn(&Device) -> H + Send + Sync + 'a,
    > Datagram<'a> for DebugSettings<'a, H, F>
{
    type O1 = DebugSettingOp<'a, H>;
    type O2 = NullOp;
    type G = DebugSettingOpGenerator<'a, H, F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(DebugSettingOpGenerator { f: self.f })
    }
}
