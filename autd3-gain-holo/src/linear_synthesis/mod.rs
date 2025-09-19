#[cfg(feature = "gs")]
mod gs;
#[cfg(feature = "gspat")]
mod gspat;
#[cfg(feature = "naive")]
mod naive;

#[cfg(feature = "gs")]
pub use gs::{GS, GSOption};
#[cfg(feature = "gspat")]
pub use gspat::{GSPAT, GSPATOption};
#[cfg(feature = "naive")]
pub use naive::{Naive, NaiveOption};
