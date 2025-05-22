use std::convert::Infallible;

use crate::firmware::{
    fpga::{GPIOOut, GPIOOutputType},
    operation::DebugSettingOp,
};

use crate::datagram::*;
use derive_more::Debug;

/// [`Datagram`] to configure GPIO Out pins for debugging.
///
/// # Example
///
/// ```
/// # use autd3_driver::datagram::GPIOOutputs;
/// # use autd3_driver::firmware::fpga::{GPIOOutputType, GPIOOut};
/// GPIOOutputs::new(|dev, gpio| match gpio {
///     GPIOOut::O0 => Some(GPIOOutputType::BaseSignal),
///     GPIOOut::O1 => Some(GPIOOutputType::Sync),
///     GPIOOut::O2 => Some(GPIOOutputType::PwmOut(&dev[0])),
///     GPIOOut::O3 => Some(GPIOOutputType::Direct(true)),
/// });
/// ```
#[derive(Debug)]
pub struct GPIOOutputs<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> {
    #[debug(ignore)]
    f: F,
}

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> GPIOOutputs<F> {
    /// Creates a new [`GPIOOutputs`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct DebugSettingOpGenerator<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync>
{
    f: F,
}

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> OperationGenerator
    for DebugSettingOpGenerator<F>
{
    type O1 = DebugSettingOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(
                [GPIOOut::O0, GPIOOut::O1, GPIOOut::O2, GPIOOut::O3]
                    .map(|gpio| (self.f)(device, gpio).into()),
            ),
            Self::O2 {},
        ))
    }
}

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> Datagram for GPIOOutputs<F> {
    type G = DebugSettingOpGenerator<F>;
    type Error = Infallible;

    fn operation_generator(self, _: &mut Geometry) -> Result<Self::G, Self::Error> {
        Ok(DebugSettingOpGenerator { f: self.f })
    }
}
