#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
/// The status of the EtherCAT device.
pub enum Status {
    /// The device is in SAFE-OP + ERROR.
    Error = 0,
    /// The device is lost.
    Lost = 1,
    /// The device is in SAFE-OP.
    StateChanged = 2,
    /// All devices resumed OP.
    Resumed = 4,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Error => write!(f, "device is in SAFE-OP + ERROR, attempting ack"),
            Status::Lost => write!(f, "device is lost"),
            Status::StateChanged => write!(f, "device is in SAFE-OP, change to OP"),
            Status::Resumed => write!(f, "all devices resumed OP"),
        }
    }
}
