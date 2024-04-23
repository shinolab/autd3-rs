pub const EC_OUTPUT_FRAME_SIZE: usize = 626;
pub const EC_INPUT_FRAME_SIZE: usize = 2;

pub const EC_CYCLE_TIME_BASE_MICRO_SEC: u64 = 500;
pub const EC_CYCLE_TIME_BASE_NANO_SEC: u64 = EC_CYCLE_TIME_BASE_MICRO_SEC * 1000;

pub const ECAT_DC_SYS_TIME_BASE: time::OffsetDateTime =
    time::macros::datetime!(2000-01-01 0:00 UTC);
