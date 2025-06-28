/// Firmware version enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Version {
    /// A driver for firmware version 10
    V10,
    /// A driver for firmware version 11
    V11,
    /// A driver for firmware version 12.0
    V12,
    /// A driver for firmware version 12.1
    V12_1,
}
