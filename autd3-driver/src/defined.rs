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

pub const FREQUENCY_40K: f64 = 40e3;
pub const ULTRASOUND_FREQUENCY: f64 = crate::firmware::fpga::FPGA_CLK_FREQ as f64 / 512.;

pub type Complex = nalgebra::Complex<f64>;

pub const ABSOLUTE_THRESHOLD_OF_HEARING: f64 = 20e-6;

pub const T4010A1_AMPLITUDE: f64 = 275.574246625 * 200.0 * MILLIMETER; // [Pa*mm]

pub const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(200);
