mod gpio_out;

pub use gpio_out::GPIOOutputType;

use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    firmware::GPIOOut,
    geometry::{Device, Geometry},
};

/// [`Datagram`] to configure GPIO Out pins.
///
/// # Example
///
/// ```
/// # use autd3_driver::datagram::GPIOOutputs;
/// # use autd3_driver::datagram::GPIOOutputType;
/// # use autd3_core::firmware::GPIOOut;
/// GPIOOutputs::new(|dev, gpio| match gpio {
///     GPIOOut::O0 => Some(GPIOOutputType::BaseSignal),
///     GPIOOut::O1 => Some(GPIOOutputType::Sync),
///     GPIOOut::O2 => Some(GPIOOutputType::PwmOut(&dev[0])),
///     GPIOOut::O3 => Some(GPIOOutputType::Direct(true)),
/// });
/// ```
pub struct GPIOOutputs<F> {
    pub(crate) f: F,
}

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> GPIOOutputs<F> {
    /// Creates a new [`GPIOOutputs`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> Datagram<'_>
    for GPIOOutputs<F>
{
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &Environment,
        _: &DeviceMask,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
