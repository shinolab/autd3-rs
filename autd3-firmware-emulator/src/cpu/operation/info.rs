use crate::{CPUEmulator, cpu::params::*};

const INFO_TYPE_CPU_VERSION_MAJOR: u8 = 0x01;
const INFO_TYPE_CPU_VERSION_MINOR: u8 = 0x02;
const INFO_TYPE_FPGA_VERSION_MAJOR: u8 = 0x03;
const INFO_TYPE_FPGA_VERSION_MINOR: u8 = 0x04;
const INFO_TYPE_FPGA_FUNCTIONS: u8 = 0x05;
const INFO_TYPE_CLEAR: u8 = 0x06;

#[repr(C, align(2))]
struct FirmInfo {
    tag: u8,
    ty: u8,
}

impl CPUEmulator {
    #[must_use]
    const fn get_cpu(&self) -> u16 {
        CPU_VERSION_MAJOR
    }

    #[must_use]
    const fn get_cpu_version_minor(&self) -> u16 {
        CPU_VERSION_MINOR
    }

    #[must_use]
    fn get_fpga(&self) -> u16 {
        self.bram_read(BRAM_SELECT_CONTROLLER, ADDR_VERSION_NUM_MAJOR)
    }

    #[must_use]
    fn get_fpga_version_minor(&self) -> u16 {
        self.bram_read(BRAM_SELECT_CONTROLLER, ADDR_VERSION_NUM_MINOR)
    }

    #[must_use]
    pub(crate) fn firm_info(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FirmInfo>(data);

        match d.ty {
            INFO_TYPE_CPU_VERSION_MAJOR => {
                self.reads_fpga_state_store = self.reads_fpga_state;
                self.reads_fpga_state = false;
                self.is_rx_data_used = true;
                self.rx_data = (self.get_cpu() & 0xFF) as _;
            }
            INFO_TYPE_CPU_VERSION_MINOR => {
                self.rx_data = (self.get_cpu_version_minor() & 0xFF) as _;
            }
            INFO_TYPE_FPGA_VERSION_MAJOR => {
                self.rx_data = (self.get_fpga() & 0xFF) as _;
            }
            INFO_TYPE_FPGA_VERSION_MINOR => {
                self.rx_data = (self.get_fpga_version_minor() & 0xFF) as _;
            }
            INFO_TYPE_FPGA_FUNCTIONS => {
                self.rx_data = ((self.get_fpga() >> 8) & 0xFF) as _;
            }
            INFO_TYPE_CLEAR => {
                self.reads_fpga_state = self.reads_fpga_state_store;
                self.is_rx_data_used = false;
            }
            _ => return ERR_INVALID_INFO_TYPE,
        };
        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_layout() {
        assert_eq!(2, std::mem::size_of::<FirmInfo>());
        assert_eq!(0, std::mem::offset_of!(FirmInfo, tag));
        assert_eq!(1, std::mem::offset_of!(FirmInfo, ty));
    }
}
