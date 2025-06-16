pub use crate::firmware::v10::fpga::{
    FOCI_STM_FIXED_NUM_UNIT, FOCI_STM_FIXED_NUM_WIDTH, FOCI_STM_FOCI_NUM_MAX,
    GAIN_STM_BUF_SIZE_MAX, PWE_BUF_SIZE,
};

/// The maximum buffer size of [`Modulation`].
///
/// [`Modulation`]: autd3_core::modulation::Modulation
pub const MOD_BUF_SIZE_MAX: usize = 65536;

/// The maximum buffer size of [`FociSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
pub const FOCI_STM_BUF_SIZE_MAX: usize = 65536;

/// The ultrasound period count bits.
pub const ULTRASOUND_PERIOD_COUNT_BITS: usize = 9;
