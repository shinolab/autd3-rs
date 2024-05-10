pub use std::f64::consts::PI;

#[cfg(feature = "use_meter")]
mod unit {
    pub const METER: f64 = 1.0;
}
#[cfg(not(feature = "use_meter"))]
mod unit {
    pub const METER: f64 = 1000.0;
}
pub use unit::*;

pub const MILLIMETER: f64 = METER / 1000.0;

pub type Complex = nalgebra::Complex<f64>;

pub const ABSOLUTE_THRESHOLD_OF_HEARING: f64 = 20e-6;

pub const T4010A1_AMPLITUDE: f64 = 275.574246625 * 200.0 * MILLIMETER; // [Pa*mm]

pub const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(200);

pub const FREQ_40K: u32 = 40000;
