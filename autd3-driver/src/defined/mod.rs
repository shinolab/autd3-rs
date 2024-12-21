mod angle;
mod freq;

pub use std::f32::consts::PI;
use std::time::Duration;

#[cfg(feature = "use_meter")]
mod unit {
    pub const METER: f32 = 1.0;
}
#[cfg(not(feature = "use_meter"))]
mod unit {
    pub const METER: f32 = 1000.0;
}
pub use unit::*;

pub use angle::*;
pub use freq::*;

pub const MILLIMETER: f32 = METER / 1000.0;

pub type Complex = nalgebra::Complex<f32>;

pub const ABSOLUTE_THRESHOLD_OF_HEARING: f32 = 20e-6;

pub const T4010A1_AMPLITUDE: f32 = 275.574_25 * 200.0 * MILLIMETER; // [Pa*mm]

pub const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(200);

pub const ULTRASOUND_FREQ: Freq<u32> = Freq { freq: 40000 };
pub const ULTRASOUND_PERIOD: Duration = Duration::from_micros(25);
pub const ULTRASOUND_PERIOD_COUNT: usize = 256;

#[allow(non_upper_case_globals)]
pub const mm: f32 = MILLIMETER;
