use autd3_derive::Builder;
use derive_more::Display;
use derive_new::new;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub struct Major(pub u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub struct Minor(pub u8);

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
        _ => format!("unknown ({major})"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Builder, new)]
#[display("{}{}", version_map(self.major, self.minor), if self.is_emulator() {" [Emulator]"} else { "" })]
pub struct FPGAVersion {
    #[get]
    major: Major,
    #[get]
    minor: Minor,
    #[get]
    function_bits: u8,
}

impl FPGAVersion {
    pub const ENABLED_EMULATOR_BIT: u8 = 1 << 7;

    pub const fn is_emulator(&self) -> bool {
        (self.function_bits & Self::ENABLED_EMULATOR_BIT) == Self::ENABLED_EMULATOR_BIT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Builder, new)]
#[display("{}", version_map(self.major, self.minor))]
pub struct CPUVersion {
    #[get]
    major: Major,
    #[get]
    minor: Minor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Builder, new)]
#[display(
    "{}: CPU = {}, FPGA = {}",
    idx,
    self.cpu(),
    self.fpga(),
)]
pub struct FirmwareVersion {
    idx: usize,
    #[get]
    cpu: CPUVersion,
    #[get]
    fpga: FPGAVersion,
}

impl FirmwareVersion {
    pub const LATEST_VERSION_NUM_MAJOR: Major = Major(0xA2);
    pub const LATEST_VERSION_NUM_MINOR: Minor = Minor(0x01);

    pub const fn is_emulator(&self) -> bool {
        self.fpga.is_emulator()
    }

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
    #[case("unknown (147)", 147)]
    fn version(#[case] expected: &str, #[case] num: u8) {
        let info = FirmwareVersion::new(
            0,
            CPUVersion::new(Major(num), Minor(0)),
            FPGAVersion::new(Major(num), Minor(0), 0),
        );
        assert_eq!(expected, info.cpu().to_string());
        assert_eq!(expected, info.fpga().to_string());
    }

    #[test]
    fn latest() {
        assert_eq!("v10.0.1", FirmwareVersion::latest());
    }

    #[test]
    fn is_emulator() {
        assert!(FirmwareVersion::new(
            0,
            CPUVersion::new(Major(0), Minor(0)),
            FPGAVersion::new(Major(0), Minor(0), FPGAVersion::ENABLED_EMULATOR_BIT)
        )
        .is_emulator());
        assert!(!FirmwareVersion::new(
            0,
            CPUVersion::new(Major(0), Minor(0)),
            FPGAVersion::new(Major(0), Minor(0), 0)
        )
        .is_emulator());
    }

    #[test]
    fn number() {
        let info = FirmwareVersion::new(
            0,
            CPUVersion::new(Major(1), Minor(3)),
            FPGAVersion::new(Major(2), Minor(4), 5),
        );
        assert_eq!(info.cpu().major().0, 1);
        assert_eq!(info.fpga().major().0, 2);
        assert_eq!(info.cpu().minor().0, 3);
        assert_eq!(info.fpga().minor().0, 4);
        assert_eq!(info.fpga().function_bits(), 5);
    }

    #[test]
    fn display() {
        let info = FirmwareVersion::new(
            0,
            CPUVersion::new(Major(1), Minor(3)),
            FPGAVersion::new(Major(2), Minor(4), 0),
        );
        assert_eq!(format!("{}", info), "0: CPU = v0.4, FPGA = v0.5");

        let info = FirmwareVersion::new(
            0,
            CPUVersion::new(Major(1), Minor(3)),
            FPGAVersion::new(Major(2), Minor(4), FPGAVersion::ENABLED_EMULATOR_BIT),
        );
        assert_eq!(format!("{}", info), "0: CPU = v0.4, FPGA = v0.5 [Emulator]");
    }
}
