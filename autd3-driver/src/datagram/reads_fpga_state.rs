use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    environment::Environment,
    firmware::FirmwareLimits,
    geometry::{Device, Geometry},
};

use derive_more::Debug;

/// [`Datagram`] to enable reading the FPGA state.
#[derive(Debug)]
pub struct ReadsFPGAState<F> {
    #[debug(ignore)]
    #[doc(hidden)]
    pub f: F,
}

impl<F: Fn(&Device) -> bool> ReadsFPGAState<F> {
    /// Creates a new [`ReadsFPGAState`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn(&Device) -> bool> Datagram<'_> for ReadsFPGAState<F> {
    type G = Self;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &Geometry,
        _: &Environment,
        _: &DeviceFilter,
        _: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        Ok(self)
    }
}
