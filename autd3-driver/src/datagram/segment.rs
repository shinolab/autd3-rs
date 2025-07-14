use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceFilter, Segment, TransitionMode},
    derive::FirmwareLimits,
    environment::Environment,
    geometry::Geometry,
};

/// [`Datagram`] to change the segment.
///
/// [`Datagram`]: autd3_core::datagram::Datagram
#[derive(Debug, Clone, Copy)]
pub enum SwapSegment {
    /// Change the [`Gain`] segment.
    ///
    /// [`Gain`]: autd3_core::gain::Gain
    Gain(Segment, TransitionMode),
    /// Change the [`Modulation`] segment.
    ///
    /// [`Modulation`]: autd3_core::modulation::Modulation
    Modulation(Segment, TransitionMode),
    /// Change the [`FociSTM`] segment.
    ///
    /// [`FociSTM`]: crate::datagram::FociSTM
    FociSTM(Segment, TransitionMode),
    /// Change the [`GainSTM`] segment.
    ///
    /// [`GainSTM`]: crate::datagram::GainSTM
    GainSTM(Segment, TransitionMode),
}

impl Datagram<'_> for SwapSegment {
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
