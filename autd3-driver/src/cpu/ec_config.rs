use time::OffsetDateTime;

use crate::derive::AUTDInternalError;

pub const ECAT_DC_SYS_TIME_BASE: time::OffsetDateTime =
    time::macros::datetime!(2000-01-01 0:00 UTC);

pub const EC_OUTPUT_FRAME_SIZE: usize = 626;
pub const EC_INPUT_FRAME_SIZE: usize = 2;

pub const EC_CYCLE_TIME_BASE_MICRO_SEC: u64 = 500;
pub const EC_CYCLE_TIME_BASE_NANO_SEC: u64 = EC_CYCLE_TIME_BASE_MICRO_SEC * 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DcSysTime {
    dc_sys_time: u64,
}

impl DcSysTime {
    pub const fn sys_time(&self) -> u64 {
        self.dc_sys_time
    }

    pub fn to_utc(&self) -> OffsetDateTime {
        ECAT_DC_SYS_TIME_BASE + std::time::Duration::from_nanos(self.dc_sys_time)
    }

    pub fn from_utc(utc: OffsetDateTime) -> Result<Self, AUTDInternalError> {
        match (utc - ECAT_DC_SYS_TIME_BASE).whole_nanoseconds() {
            i if i < 0 => Err(AUTDInternalError::InvalidDateTime),
            i => Ok(Self {
                dc_sys_time: i as u64,
            }),
        }
    }

    pub fn now() -> Self {
        Self::from_utc(OffsetDateTime::now_utc()).unwrap()
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn now_dc_sys_time() {
        let t = DcSysTime::now();
        assert!(t.sys_time() > 0);
    }

    #[test]
    fn from_utc() {
        let utc = time::macros::datetime!(2000-01-01 0:0:0 UTC);
        let t = DcSysTime::from_utc(utc);
        assert!(t.is_ok());
        let t = t.unwrap();
        assert_eq!(0, t.sys_time());
        assert_eq!(utc, t.to_utc());

        let utc = time::macros::datetime!(2000-01-01 0:0:1 UTC);
        let t = DcSysTime::from_utc(utc);
        assert!(t.is_ok());
        let t = t.unwrap();
        assert_eq!(1000000000, t.sys_time());
        assert_eq!(utc, t.to_utc());

        let utc = time::macros::datetime!(2001-01-01 0:0:0 UTC);
        let t = DcSysTime::from_utc(utc);
        assert!(t.is_ok());
        let t = t.unwrap();
        assert_eq!(31622400000000000, t.sys_time());
        assert_eq!(utc, t.to_utc());

        let utc = time::macros::datetime!(1999-01-01 0:0:1 UTC);
        let t = DcSysTime::from_utc(utc);
        assert!(t.is_err());
    }
}
