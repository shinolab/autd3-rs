/// Primitive [`Gain`]
///
/// [`Gain`]: autd3_core::gain::Gain
pub mod gain;

/// Primitive [`Modulation`]
///
/// [`Modulation`]: autd3_core::modulation::Modulation
pub mod modulation;

/// Utilities for [`GainSTM`] and [`FociSTM`]
///
/// [`GainSTM`]: autd3_driver::datagram::GainSTM
/// [`FociSTM`]: autd3_driver::datagram::FociSTM
pub mod stm;

pub use autd3_driver::datagram::BoxedDatagram;
