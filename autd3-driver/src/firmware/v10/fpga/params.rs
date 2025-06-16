use autd3_core::common::mm;

/// The unit of the fixed-point number used in the [`FociSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
pub const FOCI_STM_FIXED_NUM_UNIT: f32 = 0.025 * mm;
/// The width of the fixed-point number used in the [`FociSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
pub const FOCI_STM_FIXED_NUM_WIDTH: usize = 18;

/// The maximum buffer size of [`Modulation`].
///
/// [`Modulation`]: autd3_core::modulation::Modulation
pub const MOD_BUF_SIZE_MAX: usize = 32768;

/// The maximum number of foci.
pub const FOCI_STM_FOCI_NUM_MAX: usize = 8;
/// The maximum buffer size of [`FociSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
pub const FOCI_STM_BUF_SIZE_MAX: usize = 8192;
/// The maximum buffer size of [`GainSTM`].
///
/// [`GainSTM`]: crate::datagram::GainSTM
pub const GAIN_STM_BUF_SIZE_MAX: usize = 1024;

/// The ultrasound period count bits.
pub const ULTRASOUND_PERIOD_COUNT_BITS: usize = 8;

#[doc(hidden)]
pub const PWE_BUF_SIZE: usize = 256;
