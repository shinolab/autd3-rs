use derive_more::{Debug, Display};
use itertools::Itertools;

/// Major version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub struct Major(pub u8);

/// Minor version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub struct Minor(pub u8);

#[must_use]
fn version_map(major: Major, minor: Minor) -> String {
    let major = major.0;
    let minor = minor.0;
    match major {
        0 => "older than v0.4".to_string(),
        0x01..=0x06 => format!("v0.{}", major + 3),
        0x0A..=0x15 => format!("v1.{}", major - 0x0A),
        0x80..=0x89 => format!("v2.{}.{}", major - 0x80, minor),
        0x8A..=0x8A => format!("v3.{}.{}", major - 0x8A, minor),
        0x8B..=0x8C => format!("v4.{}.{}", major - 0x8B, minor),
        0x8D..=0x8E => format!("v5.{}.{}", major - 0x8D, minor),
        0x8F..=0x90 => format!("v6.{}.{}", major - 0x8F, minor),
        0x91..=0x91 => format!("v7.{}.{}", major - 0x91, minor),
        0x92..=0x92 => format!("v8.{}.{}", major - 0x92, minor),
        0xA0..=0xA1 => format!("v9.{}.{}", major - 0xA0, minor),
        0xA2..=0xA2 => format!("v10.{}.{}", major - 0xA2, minor),
        0xA3..=0xA3 => format!("v11.{}.{}", major - 0xA3, minor),
        0xA4..=0xA4 => format!("v12.{}.{}", major - 0xA4, minor),
        _ => format!("unknown ({major})"),
    }
}

/// FPGA firmware version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FPGAVersion {
    #[doc(hidden)]
    pub major: Major,
    #[doc(hidden)]
    pub minor: Minor,
    #[doc(hidden)]
    pub function_bits: u8,
}

impl FPGAVersion {
    #[doc(hidden)]
    pub const ENABLED_EMULATOR_BIT: u8 = 1 << 7;

    #[doc(hidden)]
    #[must_use]
    pub const fn is_emulator(&self) -> bool {
        (self.function_bits & Self::ENABLED_EMULATOR_BIT) == Self::ENABLED_EMULATOR_BIT
    }
}

impl std::fmt::Display for FPGAVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", version_map(self.major, self.minor))?;
        let features = [self.is_emulator().then_some("Emulator")]
            .iter()
            .filter_map(Option::as_ref)
            .join(", ");
        if !features.is_empty() {
            write!(f, " [{}]", features)?;
        }
        Ok(())
    }
}

/// CPU firmware version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[display("{}", version_map(self.major, self.minor))]
pub struct CPUVersion {
    #[doc(hidden)]
    pub major: Major,
    #[doc(hidden)]
    pub minor: Minor,
}

/// Firmware version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[display(
    "{}: CPU = {}, FPGA = {}",
    idx,
    self.cpu,
    self.fpga,
)]
#[debug("{}", self)]
pub struct FirmwareVersion {
    #[doc(hidden)]
    pub idx: usize,
    #[doc(hidden)]
    pub cpu: CPUVersion,
    #[doc(hidden)]
    pub fpga: FPGAVersion,
}

impl FirmwareVersion {
    #[doc(hidden)]
    pub const LATEST_VERSION_NUM_MAJOR: Major = Major(0xA4);
    #[doc(hidden)]
    pub const LATEST_VERSION_NUM_MINOR: Minor = Minor(0x00);

    #[doc(hidden)]
    #[must_use]
    pub const fn is_emulator(&self) -> bool {
        self.fpga.is_emulator()
    }

    /// Gets the latest version.
    #[must_use]
    pub fn latest() -> String {
        version_map(
            Self::LATEST_VERSION_NUM_MAJOR,
            Self::LATEST_VERSION_NUM_MINOR,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case("older than v0.4", 0)]
    #[case("v0.4", 1)]
    #[case("v0.5", 2)]
    #[case("v0.6", 3)]
    #[case("v0.7", 4)]
    #[case("v0.8", 5)]
    #[case("v0.9", 6)]
    #[case("unknown (7)", 7)]
    #[case("unknown (8)", 8)]
    #[case("unknown (9)", 9)]
    #[case("v1.0", 10)]
    #[case("v1.1", 11)]
    #[case("v1.2", 12)]
    #[case("v1.3", 13)]
    #[case("v1.4", 14)]
    #[case("v1.5", 15)]
    #[case("v1.6", 16)]
    #[case("v1.7", 17)]
    #[case("v1.8", 18)]
    #[case("v1.9", 19)]
    #[case("v1.10", 20)]
    #[case("v1.11", 21)]
    #[case("v2.0.0", 128)]
    #[case("v2.1.0", 129)]
    #[case("v2.2.0", 130)]
    #[case("v2.3.0", 131)]
    #[case("v2.4.0", 132)]
    #[case("v2.5.0", 133)]
    #[case("v2.6.0", 134)]
    #[case("v2.7.0", 135)]
    #[case("v2.8.0", 136)]
    #[case("v2.9.0", 137)]
    #[case("v3.0.0", 138)]
    #[case("v4.0.0", 139)]
    #[case("v4.1.0", 140)]
    #[case("v5.0.0", 141)]
    #[case("v5.1.0", 142)]
    #[case("v6.0.0", 143)]
    #[case("v6.1.0", 144)]
    #[case("v7.0.0", 145)]
    #[case("v8.0.0", 146)]
    #[case("v9.0.0", 160)]
    #[case("v9.1.0", 161)]
    #[case("v10.0.0", 162)]
    #[case("v11.0.0", 163)]
    #[case("v12.0.0", 164)]
    #[case("unknown (147)", 147)]
    fn version(#[case] expected: &str, #[case] num: u8) {
        let info = FirmwareVersion {
            idx: 0,
            cpu: CPUVersion {
                major: Major(num),
                minor: Minor(0),
            },
            fpga: FPGAVersion {
                major: Major(num),
                minor: Minor(0),
                function_bits: 0,
            },
        };
        assert_eq!(expected, info.cpu.to_string());
        assert_eq!(expected, info.fpga.to_string());
    }

    #[test]
    fn latest() {
        assert_eq!("v12.0.0", FirmwareVersion::latest());
    }

    #[rstest::rstest]
    #[case(false, 0)]
    #[case(true, FPGAVersion::ENABLED_EMULATOR_BIT)]
    #[test]
    fn is_emulator(#[case] expected: bool, #[case] function_bits: u8) {
        assert_eq!(
            expected,
            FirmwareVersion {
                idx: 0,
                cpu: CPUVersion {
                    major: Major(0),
                    minor: Minor(0)
                },
                fpga: FPGAVersion {
                    major: Major(0),
                    minor: Minor(0),
                    function_bits
                }
            }
            .is_emulator()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        "0: CPU = v0.4, FPGA = v0.5",
        FirmwareVersion {
            idx: 0,
            cpu: CPUVersion {
                major: Major(1),
                minor: Minor(3)
            },
            fpga: FPGAVersion {
                major: Major(2),
                minor: Minor(4),
                function_bits: 0
            }
        }
    )]
    #[case(
        "0: CPU = v0.4, FPGA = v0.5 [Emulator]",
        FirmwareVersion {
            idx: 0,
            cpu: CPUVersion {
                major: Major(1),
                minor: Minor(3)
            },
            fpga: FPGAVersion {
                major: Major(2),
                minor: Minor(4),
                function_bits: FPGAVersion::ENABLED_EMULATOR_BIT
            }
        }
    )]
    fn display(#[case] expected: &str, #[case] info: FirmwareVersion) {
        assert_eq!(expected, format!("{}", info));
        assert_eq!(expected, format!("{:?}", info));
    }
}
