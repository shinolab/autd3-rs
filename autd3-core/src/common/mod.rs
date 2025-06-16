mod angle;
mod freq;

use std::time::Duration;

pub use std::f32::consts::PI;

#[cfg(feature = "use_meter")]
mod unit {
    /// meter
    pub const METER: f32 = 1.0;
}
#[cfg(not(feature = "use_meter"))]
mod unit {
    /// meter
    pub const METER: f32 = 1000.0;
}
pub use unit::*;

pub use angle::*;
pub use freq::*;

/// millimeter
pub const MILLIMETER: f32 = METER / 1000.0;

/// The absolute threshold of hearing in \[㎩\]
pub const ABSOLUTE_THRESHOLD_OF_HEARING: f32 = 20e-6;

/// The amplitude of T4010A1 in \[㎩*mm\]
pub const T4010A1_AMPLITUDE: f32 = 275.574_25 * 200.0 * MILLIMETER; // [㎩*mm]

/// The default timeout duration
pub const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(200);

/// The frequency of ultrasound
pub const ULTRASOUND_FREQ: Freq<u32> = Freq { freq: 40000 };

/// The period of ultrasound
pub const ULTRASOUND_PERIOD: Duration = Duration::from_micros(25);

#[doc(hidden)]
pub const FOCI_STM_TR_X_MAX: i32 = 0x1AFC;
#[doc(hidden)]
pub const FOCI_STM_TR_Y_MAX: i32 = 0x14A3;

/// The minimum buffer size of modulation.
pub const MOD_BUF_SIZE_MIN: usize = 2;

/// The minimum buffer size of STM.
pub const STM_BUF_SIZE_MIN: usize = 2;

/// The minimum number of foci per pattern in FociSTM.
pub const FOCI_STM_FOCI_NUM_MIN: usize = 1;

#[doc(hidden)]
pub const SILENCER_STEPS_INTENSITY_DEFAULT: u16 = 10;
#[doc(hidden)]
pub const SILENCER_STEPS_PHASE_DEFAULT: u16 = 40;

/// \[㎜\]
#[allow(non_upper_case_globals)]
pub const mm: f32 = MILLIMETER;
