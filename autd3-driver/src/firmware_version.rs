use std::fmt;

#[derive(Debug, Clone, Copy)]
/// Firmware information
pub struct FirmwareInfo {
    idx: usize,
    cpu_version_number_major: u8,
    fpga_version_number_major: u8,
    cpu_version_number_minor: u8,
    fpga_version_number_minor: u8,
    fpga_function_bits: u8,
}

impl FirmwareInfo {
    pub const LATEST_VERSION_NUM_MAJOR: u8 = 0x8F;
    pub const LATEST_VERSION_NUM_MINOR: u8 = 0x00;

    const ENABLED_EMULATOR_BIT: u8 = 1 << 7;

    #[doc(hidden)]
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

    pub fn cpu_version(&self) -> String {
        Self::firmware_version_map(self.cpu_version_number_major, self.cpu_version_number_minor)
    }

    pub fn fpga_version(&self) -> String {
        Self::firmware_version_map(
            self.fpga_version_number_major,
            self.fpga_version_number_minor,
        )
    }

    pub const fn is_emulator(&self) -> bool {
        (self.fpga_function_bits & Self::ENABLED_EMULATOR_BIT) == Self::ENABLED_EMULATOR_BIT
    }

    fn firmware_version_map(version_number_major: u8, version_number_minor: u8) -> String {
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
            0x8F..=0x8F => format!(
                "v6.{}.{}",
                version_number_major - 0x8F,
                version_number_minor
            ),
            _ => format!("unknown ({version_number_major})"),
        }
    }

    pub fn latest_version() -> String {
        Self::firmware_version_map(
            Self::LATEST_VERSION_NUM_MAJOR,
            Self::LATEST_VERSION_NUM_MINOR,
        )
    }

    pub const fn cpu_version_number_major(&self) -> u8 {
        self.cpu_version_number_major
    }

    pub const fn cpu_version_number_minor(&self) -> u8 {
        self.cpu_version_number_minor
    }

    pub const fn fpga_version_number_major(&self) -> u8 {
        self.fpga_version_number_major
    }

    pub const fn fpga_version_number_minor(&self) -> u8 {
        self.fpga_version_number_minor
    }

    pub const fn fpga_function_bits(&self) -> u8 {
        self.fpga_function_bits
    }
}

impl fmt::Display for FirmwareInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r"{}: CPU = {}, FPGA = {}{}",
            self.idx,
            self.cpu_version(),
            self.fpga_version(),
            if self.is_emulator() {
                " [Emulator]"
            } else {
                ""
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_version() {
        let info = FirmwareInfo::new(0, 0, 0, 0, 0, 0);
        assert_eq!("older than v0.4", info.cpu_version());
        assert_eq!("older than v0.4", info.fpga_version());

        let info = FirmwareInfo::new(0, 1, 0, 1, 0, 0);
        assert_eq!("v0.4", info.cpu_version());
        assert_eq!("v0.4", info.fpga_version());

        let info = FirmwareInfo::new(0, 2, 0, 2, 0, 0);
        assert_eq!("v0.5", info.cpu_version());
        assert_eq!("v0.5", info.fpga_version());

        let info = FirmwareInfo::new(0, 3, 0, 3, 0, 0);
        assert_eq!("v0.6", info.cpu_version());
        assert_eq!("v0.6", info.fpga_version());

        let info = FirmwareInfo::new(0, 4, 0, 4, 0, 0);
        assert_eq!("v0.7", info.cpu_version());
        assert_eq!("v0.7", info.fpga_version());

        let info = FirmwareInfo::new(0, 5, 0, 5, 0, 0);
        assert_eq!("v0.8", info.cpu_version());
        assert_eq!("v0.8", info.fpga_version());

        let info = FirmwareInfo::new(0, 6, 0, 6, 0, 0);
        assert_eq!("v0.9", info.cpu_version());
        assert_eq!("v0.9", info.fpga_version());

        let info = FirmwareInfo::new(0, 7, 0, 7, 0, 0);
        assert_eq!("unknown (7)", info.cpu_version());
        assert_eq!("unknown (7)", info.fpga_version());

        let info = FirmwareInfo::new(0, 8, 0, 8, 0, 0);
        assert_eq!("unknown (8)", info.cpu_version());
        assert_eq!("unknown (8)", info.fpga_version());

        let info = FirmwareInfo::new(0, 9, 0, 9, 0, 0);
        assert_eq!("unknown (9)", info.cpu_version());
        assert_eq!("unknown (9)", info.fpga_version());

        let info = FirmwareInfo::new(0, 10, 0, 10, 0, 0);
        assert_eq!("v1.0", info.cpu_version());
        assert_eq!("v1.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 11, 0, 11, 0, 0);
        assert_eq!("v1.1", info.cpu_version());
        assert_eq!("v1.1", info.fpga_version());

        let info = FirmwareInfo::new(0, 12, 0, 12, 0, 0);
        assert_eq!("v1.2", info.cpu_version());
        assert_eq!("v1.2", info.fpga_version());

        let info = FirmwareInfo::new(0, 13, 0, 13, 0, 0);
        assert_eq!("v1.3", info.cpu_version());
        assert_eq!("v1.3", info.fpga_version());

        let info = FirmwareInfo::new(0, 14, 0, 14, 0, 0);
        assert_eq!("v1.4", info.cpu_version());
        assert_eq!("v1.4", info.fpga_version());

        let info = FirmwareInfo::new(0, 15, 0, 15, 0, 0);
        assert_eq!("v1.5", info.cpu_version());
        assert_eq!("v1.5", info.fpga_version());

        let info = FirmwareInfo::new(0, 16, 0, 16, 0, 0);
        assert_eq!("v1.6", info.cpu_version());
        assert_eq!("v1.6", info.fpga_version());

        let info = FirmwareInfo::new(0, 17, 0, 17, 0, 0);
        assert_eq!("v1.7", info.cpu_version());
        assert_eq!("v1.7", info.fpga_version());

        let info = FirmwareInfo::new(0, 18, 0, 18, 0, 0);
        assert_eq!("v1.8", info.cpu_version());
        assert_eq!("v1.8", info.fpga_version());

        let info = FirmwareInfo::new(0, 19, 0, 19, 0, 0);
        assert_eq!("v1.9", info.cpu_version());
        assert_eq!("v1.9", info.fpga_version());

        let info = FirmwareInfo::new(0, 20, 0, 20, 0, 0);
        assert_eq!("v1.10", info.cpu_version());
        assert_eq!("v1.10", info.fpga_version());

        let info = FirmwareInfo::new(0, 21, 0, 21, 0, 0);
        assert_eq!("v1.11", info.cpu_version());
        assert_eq!("v1.11", info.fpga_version());

        let info = FirmwareInfo::new(0, 128, 0, 128, 0, 0);
        assert_eq!("v2.0.0", info.cpu_version());
        assert_eq!("v2.0.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 129, 0, 129, 0, 0);
        assert_eq!("v2.1.0", info.cpu_version());
        assert_eq!("v2.1.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 130, 0, 130, 0, 0);
        assert_eq!("v2.2.0", info.cpu_version());
        assert_eq!("v2.2.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 131, 0, 131, 0, 0);
        assert_eq!("v2.3.0", info.cpu_version());
        assert_eq!("v2.3.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 132, 0, 132, 0, 0);
        assert_eq!("v2.4.0", info.cpu_version());
        assert_eq!("v2.4.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 133, 0, 133, 0, 0);
        assert_eq!("v2.5.0", info.cpu_version());
        assert_eq!("v2.5.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 134, 0, 134, 0, 0);
        assert_eq!("v2.6.0", info.cpu_version());
        assert_eq!("v2.6.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 135, 0, 135, 0, 0);
        assert_eq!("v2.7.0", info.cpu_version());
        assert_eq!("v2.7.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 136, 0, 136, 0, 0);
        assert_eq!("v2.8.0", info.cpu_version());
        assert_eq!("v2.8.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 136, 1, 136, 1, 0);
        assert_eq!("v2.8.1", info.cpu_version());
        assert_eq!("v2.8.1", info.fpga_version());

        let info = FirmwareInfo::new(0, 137, 0, 137, 0, 0);
        assert_eq!("v2.9.0", info.cpu_version());
        assert_eq!("v2.9.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 138, 0, 138, 0, 0);
        assert_eq!("v3.0.0", info.cpu_version());
        assert_eq!("v3.0.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 139, 0, 139, 0, 0);
        assert_eq!("v4.0.0", info.cpu_version());
        assert_eq!("v4.0.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 140, 0, 140, 0, 0);
        assert_eq!("v4.1.0", info.cpu_version());
        assert_eq!("v4.1.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 141, 0, 141, 0, 0);
        assert_eq!("v5.0.0", info.cpu_version());
        assert_eq!("v5.0.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 142, 0, 142, 0, 0);
        assert_eq!("v5.1.0", info.cpu_version());
        assert_eq!("v5.1.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 143, 0, 143, 0, 0);
        assert_eq!("v6.0.0", info.cpu_version());
        assert_eq!("v6.0.0", info.fpga_version());

        let info = FirmwareInfo::new(0, 144, 0, 144, 0, 0);
        assert_eq!("unknown (144)", info.cpu_version());
        assert_eq!("unknown (144)", info.fpga_version());
    }

    #[test]
    fn latest_firmware_version() {
        assert_eq!("v6.0.0", FirmwareInfo::latest_version());
    }

    #[test]
    fn is_emulator() {
        assert!(FirmwareInfo::new(0, 0, 0, 0, 0, FirmwareInfo::ENABLED_EMULATOR_BIT).is_emulator());
        assert!(!FirmwareInfo::new(0, 0, 0, 0, 0, 0).is_emulator());
    }

    #[test]
    fn number() {
        let info = FirmwareInfo::new(0, 1, 2, 3, 4, 5);
        assert_eq!(info.cpu_version_number_major(), 1);
        assert_eq!(info.cpu_version_number_minor(), 2);
        assert_eq!(info.fpga_version_number_major(), 3);
        assert_eq!(info.fpga_version_number_minor(), 4);
        assert_eq!(info.fpga_function_bits(), 5);
    }

    #[test]
    fn fmt() {
        let info = FirmwareInfo::new(0, 1, 2, 3, 4, 0);
        assert_eq!(format!("{}", info), "0: CPU = v0.4, FPGA = v0.6");

        let info = FirmwareInfo::new(0, 1, 2, 3, 4, FirmwareInfo::ENABLED_EMULATOR_BIT);
        assert_eq!(format!("{}", info), "0: CPU = v0.4, FPGA = v0.6 [Emulator]");
    }
}
