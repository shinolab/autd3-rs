use time::OffsetDateTime;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
#[error("Invalid date time")]
pub struct InvalidDateTime;

use super::ECAT_DC_SYS_TIME_BASE;

/// The system time of the Distributed Clock
///
/// The system time is the time expressed in 1ns units with 2000-01-01 0:00:00 UTC as the reference.
/// It is expressed as a 64-bit unsigned integer and can represent about 584 years of time.
/// See [EtherCAT Distributed Clock](https://infosys.beckhoff.com/english.php?content=../content/1033/ethercatsystem/2469118347.html) for more information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct DcSysTime {
    dc_sys_time: u64,
}

impl DcSysTime {
    /// The zero point of the DcSysTime (2000-01-01 0:00:00 UTC)
    pub const ZERO: Self = Self { dc_sys_time: 0 };

    /// Returns the system time in nanoseconds
    #[must_use]
    pub const fn sys_time(&self) -> u64 {
        self.dc_sys_time
    }

    /// Converts the system time to the UTC time
    #[must_use]
    pub fn to_utc(&self) -> OffsetDateTime {
        ECAT_DC_SYS_TIME_BASE + std::time::Duration::from_nanos(self.dc_sys_time)
    }

    /// Creates a new instance from the UTC time
    pub fn from_utc(utc: OffsetDateTime) -> Result<Self, InvalidDateTime> {
        Ok(Self {
            dc_sys_time: u64::try_from((utc - ECAT_DC_SYS_TIME_BASE).whole_nanoseconds())
                .map_err(|_| InvalidDateTime)?,
        })
    }

    /// Returns the system time of now
    #[must_use]
    pub fn now() -> Self {
        Self::from_utc(OffsetDateTime::now_utc()).unwrap()
    }
}

impl std::ops::Add<std::time::Duration> for DcSysTime {
    type Output = Self;

    fn add(self, rhs: std::time::Duration) -> Self::Output {
        Self {
            dc_sys_time: self.dc_sys_time + rhs.as_nanos() as u64,
        }
    }
}

impl std::ops::Sub<std::time::Duration> for DcSysTime {
    type Output = Self;

    fn sub(self, rhs: std::time::Duration) -> Self::Output {
        Self {
            dc_sys_time: self.dc_sys_time - rhs.as_nanos() as u64,
        }
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

    #[rstest::rstest]
    #[test]
    #[case(Ok(DcSysTime { dc_sys_time: 0 }), time::macros::datetime!(2000-01-01 0:0:0 UTC))]
    #[case(Ok(DcSysTime { dc_sys_time: 1000000000 }), time::macros::datetime!(2000-01-01 0:0:1 UTC))]
    #[case(Ok(DcSysTime { dc_sys_time: 31622400000000000 }), time::macros::datetime!(2001-01-01 0:0:0 UTC))]
    #[case(Err(InvalidDateTime), time::macros::datetime!(1999-01-01 0:0:1 UTC))]
    #[case(Err(InvalidDateTime), time::macros::datetime!(9999-01-01 0:0:1 UTC))]
    fn from_utc(#[case] expect: Result<DcSysTime, InvalidDateTime>, #[case] utc: OffsetDateTime) {
        assert_eq!(expect, DcSysTime::from_utc(utc));
    }

    #[rstest::rstest]
    #[test]
    #[case(time::macros::datetime!(2000-01-01 0:0:1 UTC))]
    #[case(time::macros::datetime!(2001-01-01 0:0:0 UTC))]
    fn to_utc(#[case] utc: OffsetDateTime) {
        assert_eq!(utc, DcSysTime::from_utc(utc).unwrap().to_utc());
    }

    #[test]
    fn add_sub() {
        let utc = time::macros::datetime!(2000-01-01 0:0:0 UTC);
        let t = DcSysTime::from_utc(utc);
        assert!(t.is_ok());
        let t = t.unwrap() + std::time::Duration::from_secs(1);
        assert_eq!(1000000000, t.sys_time());

        let t = t - std::time::Duration::from_secs(1);
        assert_eq!(0, t.sys_time());
    }
}
