mod dc_sys_time;

pub use dc_sys_time::DcSysTime;

use core::time::Duration;

/// PDO output frame size
pub const EC_OUTPUT_FRAME_SIZE: usize = 626;
/// PDO input frame size
pub const EC_INPUT_FRAME_SIZE: usize = 2;

/// The base unit of the EtherCAT
pub const EC_CYCLE_TIME_BASE: Duration = Duration::from_micros(500);

/// The base point of system time
#[cfg(feature = "time")]
pub const ECAT_DC_SYS_TIME_BASE: time::OffsetDateTime =
    time::macros::datetime!(2000-01-01 0:00 UTC);
