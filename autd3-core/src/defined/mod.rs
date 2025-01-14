mod angle;
mod freq;

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

/// The period of ultrasound in discrete time units
pub const ULTRASOUND_PERIOD_COUNT: usize = 256;

#[cfg(not(feature = "dynamic_freq"))]
mod inner {
    use super::Freq;
    use std::time::Duration;

    #[inline(always)]
    /// The frequency of ultrasound
    pub const fn ultrasound_freq() -> Freq<u32> {
        Freq { freq: 40000 }
    }

    #[inline(always)]
    /// The period of ultrasound
    pub const fn ultrasound_period() -> Duration {
        Duration::from_micros(25)
    }
}

#[cfg(feature = "dynamic_freq")]
mod inner {
    use std::sync::Once;

    use super::Freq;
    use crate::defined::Hz;

    static mut VAL: Freq<u32> = Freq { freq: 40000 };
    static FREQ: Once = Once::new();

    #[inline]
    /// The frequency of ultrasound
    pub fn ultrasound_freq() -> Freq<u32> {
        unsafe {
            FREQ.call_once(|| {
                VAL = match std::env::var("AUTD3_ULTRASOUND_FREQ") {
                    Ok(freq) => match freq.parse::<u32>() {
                        Ok(freq) => {
                            tracing::info!("Set ultrasound frequency to {} Hz.", freq);
                            freq * Hz
                        }
                        Err(_) => {
                            tracing::error!(
                                "Invalid ultrasound frequency ({} Hz), fallback to 40 kHz.",
                                freq
                            );
                            Freq { freq: 40000 }
                        }
                    },
                    Err(_) => {
                        tracing::warn!("Environment variable AUTD3_ULTRASOUND_FREQ is not set, fallback to 40 kHz.");
                        Freq { freq: 40000 }
                    }
                };
            });
            VAL
        }
    }

    #[doc(hidden)]
    pub const DRP_ROM_SIZE: usize = 32;
}

pub use inner::*;

/// \[㎜\]
#[allow(non_upper_case_globals)]
pub const mm: f32 = MILLIMETER;
