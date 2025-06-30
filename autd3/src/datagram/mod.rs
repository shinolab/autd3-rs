#[cfg_attr(docsrs, doc(cfg(feature = "gain")))]
#[cfg(feature = "gain")]
/// Primitive [`Gain`]s
///
/// [`Gain`]: autd3_core::gain::Gain
pub mod gain;

#[cfg_attr(docsrs, doc(cfg(feature = "modulation")))]
#[cfg(feature = "modulation")]
/// Primitive [`Modulation`]s
///
/// [`Modulation`]: autd3_core::modulation::Modulation
pub mod modulation;

#[cfg_attr(docsrs, doc(cfg(feature = "stm")))]
#[cfg(feature = "stm")]
/// Utilities for [`GainSTM`] and [`FociSTM`]
///
/// [`GainSTM`]: autd3_driver::datagram::GainSTM
/// [`FociSTM`]: autd3_driver::datagram::FociSTM
pub mod stm;
