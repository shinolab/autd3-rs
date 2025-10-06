use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DeviceMask},
    environment::Environment,
    firmware::{Segment, transition_mode::TransitionMode},
    geometry::Geometry,
};

/// Change the [`Gain`] segment.
///
/// [`Gain`]: autd3_core::gain::Gain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapSegmentGain(pub Segment);

/// Change the [`Modulation`] segment.
///    
/// [`Modulation`]: autd3_core::modulation::Modulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapSegmentModulation<T>(pub Segment, pub T);

/// Change the [`FociSTM`] segment.
///
/// [`FociSTM`]: crate::datagram::FociSTM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapSegmentFociSTM<T>(pub Segment, pub T);

/// Change the [`GainSTM`] segment.
///
/// [`GainSTM`]: crate::datagram::GainSTM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapSegmentGainSTM<T>(pub Segment, pub T);

impl Datagram<'_> for SwapSegmentGain {
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

impl<T: TransitionMode> Datagram<'_> for SwapSegmentModulation<T> {
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

impl<T: TransitionMode> Datagram<'_> for SwapSegmentFociSTM<T> {
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

impl<T: TransitionMode> Datagram<'_> for SwapSegmentGainSTM<T> {
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
