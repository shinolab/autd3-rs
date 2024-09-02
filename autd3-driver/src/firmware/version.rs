use autd3_derive::Builder;
use derive_more::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Builder)]
#[display(
    "{}: CPU = {}, FPGA = {}{}",
    idx,
    self.cpu(),
    self.fpga(),
    if self.is_emulator() {" [Emulator]"} else { "" }
)]
pub struct FirmwareVersion {
    idx: usize,
    #[get]
    cpu_version_number_major: u8,
    #[get]
    fpga_version_number_major: u8,
    #[get]
    cpu_version_number_minor: u8,
    #[get]
    fpga_version_number_minor: u8,
    #[get]
    fpga_function_bits: u8,
}

impl FirmwareVersion {
    pub const LATEST_VERSION_NUM_MAJOR: u8 = 0xA2;
    pub const LATEST_VERSION_NUM_MINOR: u8 = 0x00;
    pub const ENABLED_EMULATOR_BIT: u8 = 1 << 7;

    pub const fn new(
        idx: usize,
        cpu_version_number_major: u8,
        cpu_version_number_minor: u8,
        fpga_version_number_major: u8,
        fpga_version_number_minor: u8,
        fpga_function_bits: u8,
    ) -> Self {
        Self {
            idx,
            cpu_version_number_major,
            fpga_version_number_major,
            cpu_version_number_minor,
            fpga_version_number_minor,
            fpga_function_bits,
        }
    }

    pub fn cpu(&self) -> String {
        Self::version_map(self.cpu_version_number_major, self.cpu_version_number_minor)
    }

    pub fn fpga(&self) -> String {
        Self::version_map(
            self.fpga_version_number_major,
            self.fpga_version_number_minor,
        )
    }

    pub const fn is_emulator(&self) -> bool {
        (self.fpga_function_bits & Self::ENABLED_EMULATOR_BIT) == Self::ENABLED_EMULATOR_BIT
    }

    fn version_map(version_number_major: u8, version_number_minor: u8) -> String {
        match version_number_major {
            0 => "older than v0.4".to_string(),
            0x01..=0x06 => format!("v0.{}", version_number_major + 3),
            0x0A..=0x15 => format!("v1.{}", version_number_major - 0x0A),
            0x80..=0x89 => format!(
                "v2.{}.{}",
                version_number_major - 0x80,
                version_number_minor
            ),
            0x8A..=0x8A => format!(
                "v3.{}.{}",
                version_number_major - 0x8A,
                version_number_minor
            ),
            0x8B..=0x8C => format!(
                "v4.{}.{}",
                version_number_major - 0x8B,
                version_number_minor
            ),
            0x8D..=0x8E => format!(
                "v5.{}.{}",
                version_number_major - 0x8D,
                version_number_minor
            ),
            0x8F..=0x90 => format!(
                "v6.{}.{}",
                version_number_major - 0x8F,
                version_number_minor
            ),
            0x91..=0x91 => format!(
                "v7.{}.{}",
                version_number_major - 0x91,
                version_number_minor
            ),
            0x92..=0x92 => format!(
                "v8.{}.{}",
                version_number_major - 0x92,
                version_number_minor
            ),
            0xA0..=0xA1 => format!(
                "v9.{}.{}",
                version_number_major - 0xA0,
                version_number_minor
            ),
            0xA2..=0xA2 => format!(
                "v10.{}.{}",
                version_number_major - 0xA2,
                version_number_minor
            ),
            _ => format!("unknown ({version_number_major})"),
        }
    }

    pub fn latest() -> String {
        Self::version_map(
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
    #[case("unknown (147)", 147)]
    #[cfg_attr(miri, ignore)]
    fn version(#[case] expected: &str, #[case] num: u8) {
        let info = FirmwareVersion::new(0, num, 0, num, 0, 0);
        assert_eq!(expected, info.cpu());
        assert_eq!(expected, info.fpga());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn latest() {
        assert_eq!("v10.0.0", FirmwareVersion::latest());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn is_emulator() {
        assert!(
            FirmwareVersion::new(0, 0, 0, 0, 0, FirmwareVersion::ENABLED_EMULATOR_BIT)
                .is_emulator()
        );
        assert!(!FirmwareVersion::new(0, 0, 0, 0, 0, 0).is_emulator());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn number() {
        let info = FirmwareVersion::new(0, 1, 2, 3, 4, 5);
        assert_eq!(info.cpu_version_number_major(), 1);
        assert_eq!(info.cpu_version_number_minor(), 2);
        assert_eq!(info.fpga_version_number_major(), 3);
        assert_eq!(info.fpga_version_number_minor(), 4);
        assert_eq!(info.fpga_function_bits(), 5);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn display() {
        let info = FirmwareVersion::new(0, 1, 2, 3, 4, 0);
        assert_eq!(format!("{}", info), "0: CPU = v0.4, FPGA = v0.6");

        let info = FirmwareVersion::new(0, 1, 2, 3, 4, FirmwareVersion::ENABLED_EMULATOR_BIT);
        assert_eq!(format!("{}", info), "0: CPU = v0.4, FPGA = v0.6 [Emulator]");
    }
}
