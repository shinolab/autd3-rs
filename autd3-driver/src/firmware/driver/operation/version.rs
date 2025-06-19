/// Firmware version enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    /// A driver for firmware version 10.
    V10,
    /// A driver for firmware version 11.
    V11,
    /// A driver for firmware version 12.
    V12,
}
