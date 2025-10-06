#[cfg(feature = "time")]
#[derive(Debug, PartialEq, Clone)]
pub struct InvalidDateTime;

#[cfg(feature = "time")]
impl core::fmt::Display for InvalidDateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid date time")
    }
}

#[cfg(all(feature = "time", feature = "std"))]
impl std::error::Error for InvalidDateTime {}

#[cfg(feature = "time")]
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

    /// Creates a new instance with the given system time in nanoseconds since 2000-01-01 0:00:00 UTC
    pub const fn new(dc_sys_time: u64) -> Self {
        Self { dc_sys_time }
    }

    /// Returns the system time in nanoseconds
    #[must_use]
    pub const fn sys_time(&self) -> u64 {
        self.dc_sys_time
    }

    /// Converts the system time to the UTC time
    #[must_use]
    #[cfg(feature = "time")]
    pub fn to_utc(&self) -> time::OffsetDateTime {
        ECAT_DC_SYS_TIME_BASE + core::time::Duration::from_nanos(self.dc_sys_time)
    }

    /// Creates a new instance from the UTC time
    #[cfg(feature = "time")]
    pub fn from_utc(utc: time::OffsetDateTime) -> Result<Self, InvalidDateTime> {
        Ok(Self {
            dc_sys_time: u64::try_from((utc - ECAT_DC_SYS_TIME_BASE).whole_nanoseconds())
                .map_err(|_| InvalidDateTime)?,
        })
    }

    /// Returns the system time of now
    #[must_use]
    #[cfg(all(feature = "time", feature = "std"))]
    pub fn now() -> Self {
        Self::from_utc(time::OffsetDateTime::now_utc()).unwrap()
    }
}

impl core::ops::Add<core::time::Duration> for DcSysTime {
    type Output = Self;

    fn add(self, rhs: core::time::Duration) -> Self::Output {
        Self {
            dc_sys_time: self.dc_sys_time + rhs.as_nanos() as u64,
        }
    }
}

impl core::ops::Sub<core::time::Duration> for DcSysTime {
    type Output = Self;

    fn sub(self, rhs: core::time::Duration) -> Self::Output {
        Self {
            dc_sys_time: self.dc_sys_time - rhs.as_nanos() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let t = DcSysTime::new(123456789);
        assert_eq!(123456789, t.sys_time());
    }

    #[cfg(feature = "time")]
    #[test]
    fn now_dc_sys_time() {
        let t = DcSysTime::now();
        assert!(t.sys_time() > 0);
    }

    #[cfg(feature = "time")]
    #[rstest::rstest]
    #[case(Ok(DcSysTime { dc_sys_time: 0 }), time::macros::datetime!(2000-01-01 0:0:0 UTC))]
    #[case(Ok(DcSysTime { dc_sys_time: 1000000000 }), time::macros::datetime!(2000-01-01 0:0:1 UTC))]
    #[case(Ok(DcSysTime { dc_sys_time: 31622400000000000 }), time::macros::datetime!(2001-01-01 0:0:0 UTC))]
    #[case(Err(InvalidDateTime), time::macros::datetime!(1999-01-01 0:0:1 UTC))]
    #[case(Err(InvalidDateTime), time::macros::datetime!(9999-01-01 0:0:1 UTC))]
    fn from_utc(
        #[case] expect: Result<DcSysTime, InvalidDateTime>,
        #[case] utc: time::OffsetDateTime,
    ) {
        assert_eq!(expect, DcSysTime::from_utc(utc));
    }

    #[cfg(feature = "time")]
    #[rstest::rstest]
    #[case(time::macros::datetime!(2000-01-01 0:0:1 UTC))]
    #[case(time::macros::datetime!(2001-01-01 0:0:0 UTC))]
    fn to_utc(#[case] utc: time::OffsetDateTime) {
        assert_eq!(utc, DcSysTime::from_utc(utc).unwrap().to_utc());
    }

    #[cfg(feature = "time")]
    #[test]
    fn add_sub() {
        let utc = time::macros::datetime!(2000-01-01 0:0:0 UTC);
        let t = DcSysTime::from_utc(utc);
        assert!(t.is_ok());
        let t = t.unwrap() + core::time::Duration::from_secs(1);
        assert_eq!(1000000000, t.sys_time());

        let t = t - core::time::Duration::from_secs(1);
        assert_eq!(0, t.sys_time());
    }
}
