#[cfg(any(
    feature = "defined",
    feature = "geometry",
    feature = "gain",
    feature = "modulation",
    feature = "link"
))]
/// Common constants and types.
pub mod defined;
#[cfg(any(feature = "ethercat", feature = "link"))]
pub mod ethercat;
#[cfg(feature = "gain")]
pub mod gain;
#[cfg(any(feature = "geometry", feature = "gain", feature = "link"))]
/// Geometry related modules.
pub mod geometry;
#[cfg(feature = "link")]
/// A interface to the device.
pub mod link;
#[cfg(feature = "modulation")]
pub mod modulation;
#[cfg(any(feature = "utils", feature = "modulation"))]
#[doc(hidden)]
pub mod utils;
