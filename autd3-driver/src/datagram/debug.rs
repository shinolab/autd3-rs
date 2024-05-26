use crate::{
    datagram::*,
    derive::DEFAULT_TIMEOUT,
    firmware::fpga::{DebugType, GPIOOut},
    geometry::Device,
};

pub struct DebugSettings<'a, H: Fn(GPIOOut) -> DebugType<'a>, F: Fn(&'a Device) -> H + Send + Sync>
{
    f: F,
    _phantom: std::marker::PhantomData<&'a H>,
}

impl<'a, H: Fn(GPIOOut) -> DebugType<'a>, F: Fn(&'a Device) -> H + Send + Sync>
    DebugSettings<'a, H, F>
{
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, H: Fn(GPIOOut) -> DebugType<'a>, F: Fn(&'a Device) -> H + Send + Sync> Datagram<'a>
    for DebugSettings<'a, H, F>
{
    type O1 = crate::firmware::operation::DebugSettingOp<'a, H>;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(
        &'a self,
        _: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        let f = &self.f;
        Ok(|dev| (Self::O1::new(f(dev)), Self::O2::default()))
    }
}
