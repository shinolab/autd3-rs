/*
 * File: info.rs
 * Project: operation
 * Created Date: 30/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use crate::{
    cpu::params::{
        BRAM_ADDR_VERSION_NUM, BRAM_ADDR_VERSION_NUM_MINOR, BRAM_SELECT_CONTROLLER,
        CPU_VERSION_MAJOR, CPU_VERSION_MINOR, ERR_NONE,
    },
    CPUEmulator,
};

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
    fn get_cpu_version(&self) -> u16 {
        CPU_VERSION_MAJOR
    }

    fn get_cpu_version_minor(&self) -> u16 {
        CPU_VERSION_MINOR
    }

    fn get_fpga_version(&self) -> u16 {
        self.bram_read(BRAM_SELECT_CONTROLLER, BRAM_ADDR_VERSION_NUM)
    }

    fn get_fpga_version_minor(&self) -> u16 {
        self.bram_read(BRAM_SELECT_CONTROLLER, BRAM_ADDR_VERSION_NUM_MINOR)
    }

    pub(crate) fn firm_info(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FirmInfo>(data);

        match d.ty {
            INFO_TYPE_CPU_VERSION_MAJOR => {
                self.read_fpga_info_store = self.read_fpga_info;
                self.read_fpga_info = false;
                self.rx_data = (self.get_cpu_version() & 0xFF) as _;
            }
            INFO_TYPE_CPU_VERSION_MINOR => {
                self.rx_data = (self.get_cpu_version_minor() & 0xFF) as _;
            }
            INFO_TYPE_FPGA_VERSION_MAJOR => {
                self.rx_data = (self.get_fpga_version() & 0xFF) as _;
            }
            INFO_TYPE_FPGA_VERSION_MINOR => {
                self.rx_data = (self.get_fpga_version_minor() & 0xFF) as _;
            }
            INFO_TYPE_FPGA_FUNCTIONS => {
                self.rx_data = ((self.get_fpga_version() >> 8) & 0xFF) as _;
            }
            INFO_TYPE_CLEAR => {
                self.read_fpga_info = self.read_fpga_info_store;
            }
            _ => {
                unreachable!("Unsupported firmware info type")
            }
        };
        ERR_NONE
    }
}
