/*
 * File: emulator.rs
 * Project: src
 * Created Date: 06/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 19/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::cpu::{Header, TxDatagram};

use crate::fpga::emulator::FPGAEmulator;

use super::{operation::*, params::*};

pub struct CPUEmulator {
    pub(crate) idx: usize,
    pub(crate) ack: u8,
    pub(crate) last_msg_id: u8,
    pub(crate) rx_data: u8,
    pub(crate) read_fpga_state: bool,
    pub(crate) read_fpga_state_store: bool,
    pub(crate) mod_cycle: u32,
    pub(crate) stm_cycle: u32,
    pub(crate) gain_stm_mode: u16,
    pub(crate) fpga: FPGAEmulator,
    pub(crate) synchronized: bool,
    pub(crate) num_transducers: usize,
    pub(crate) fpga_flags_internal: u16,
    pub(crate) mod_freq_div: u32,
    pub(crate) stm_freq_div: u32,
    pub(crate) silencer_strict_mode: bool,
    pub(crate) min_freq_div_intensity: u32,
    pub(crate) min_freq_div_phase: u32,
    pub(crate) is_rx_data_used: bool,
}

impl CPUEmulator {
    pub fn new(id: usize, num_transducers: usize) -> Self {
        let mut s = Self {
            idx: id,
            ack: 0x00,
            last_msg_id: 0x00,
            rx_data: 0x00,
            read_fpga_state: false,
            read_fpga_state_store: false,
            mod_cycle: 0,
            stm_cycle: 0,
            gain_stm_mode: 0,
            fpga: FPGAEmulator::new(num_transducers),
            synchronized: false,
            num_transducers,
            fpga_flags_internal: 0x0000,
            mod_freq_div: 5120,
            stm_freq_div: 0xFFFFFFFF,
            silencer_strict_mode: true,
            min_freq_div_intensity: 5120,
            min_freq_div_phase: 20480,
            is_rx_data_used: false,
        };
        s.init();
        s
    }

    pub const fn idx(&self) -> usize {
        self.idx
    }

    pub const fn num_transducers(&self) -> usize {
        self.num_transducers
    }

    pub const fn synchronized(&self) -> bool {
        self.synchronized
    }

    pub const fn reads_fpga_state(&self) -> bool {
        self.read_fpga_state
    }

    pub const fn ack(&self) -> u8 {
        self.ack
    }

    pub const fn rx_data(&self) -> u8 {
        self.rx_data
    }

    pub const fn fpga(&self) -> &FPGAEmulator {
        &self.fpga
    }

    pub fn fpga_mut(&mut self) -> &mut FPGAEmulator {
        &mut self.fpga
    }

    pub fn send(&mut self, tx: &TxDatagram) {
        self.ecat_recv(tx.data(self.idx));
    }

    pub fn init(&mut self) {
        self.fpga.init();
        self.clear(&[]);
    }

    pub fn update(&mut self) {
        if self.should_update() {
            self.read_fpga_state();
        }
    }

    pub const fn should_update(&self) -> bool {
        self.read_fpga_state
    }

    pub const fn silencer_strict_mode(&self) -> bool {
        self.silencer_strict_mode
    }
}

impl CPUEmulator {
    pub(crate) const fn cast<T>(data: &[u8]) -> &T {
        unsafe { &*(data.as_ptr() as *const T) }
    }

    const fn get_addr(select: u8, addr: u16) -> u16 {
        ((select as u16 & 0x0003) << 14) | (addr & 0x3FFF)
    }

    pub(crate) fn bram_read(&self, select: u8, addr: u16) -> u16 {
        let addr = Self::get_addr(select, addr);
        self.fpga.read(addr)
    }

    pub(crate) fn bram_write(&mut self, select: u8, addr: u16, data: u16) {
        let addr = Self::get_addr(select, addr);
        self.fpga.write(addr, data)
    }

    pub(crate) fn bram_cpy(&mut self, select: u8, addr_base: u16, data: *const u16, size: usize) {
        let mut addr = Self::get_addr(select, addr_base);
        let mut src = data;
        (0..size).for_each(|_| unsafe {
            self.fpga.write(addr, src.read());
            addr += 1;
            src = src.add(1);
        })
    }

    pub(crate) fn bram_set(&mut self, select: u8, addr_base: u16, value: u16, size: usize) {
        let mut addr = Self::get_addr(select, addr_base);
        (0..size).for_each(|_| {
            self.fpga.write(addr, value);
            addr += 1;
        })
    }

    fn read_fpga_state(&mut self) {
        if self.is_rx_data_used {
            return;
        }
        if self.read_fpga_state {
            self.rx_data = READS_FPGA_STATE_ENABLED
                | self.bram_read(BRAM_SELECT_CONTROLLER, BRAM_ADDR_FPGA_STATE) as u8;
        } else {
            self.rx_data &= !READS_FPGA_STATE_ENABLED;
        }
    }

    fn handle_payload(&mut self, data: &[u8]) -> u8 {
        unsafe {
            match data[0] {
                TAG_CLEAR => self.clear(data),
                TAG_SYNC => self.synchronize(data),
                TAG_FIRM_INFO => self.firm_info(data),
                TAG_MODULATION => self.write_mod(data),
                TAG_MODULATION_DELAY => self.write_mod_delay(data),
                TAG_SILENCER => self.config_silencer(data),
                TAG_GAIN => self.write_gain(data),
                TAG_FOCUS_STM => self.write_focus_stm(data),
                TAG_GAIN_STM => self.write_gain_stm(data),
                TAG_FORCE_FAN => self.configure_force_fan(data),
                TAG_READS_FPGA_STATE => self.configure_reads_fpga_state(data),
                TAG_DEBUG => self.config_debug(data),
                _ => ERR_NOT_SUPPORTED_TAG,
            }
        }
    }

    fn ecat_recv(&mut self, data: &[u8]) {
        let header = unsafe { &*(data.as_ptr() as *const Header) };

        if self.ack == header.msg_id {
            return;
        }
        self.last_msg_id = header.msg_id;

        self.read_fpga_state();

        if (header.msg_id & 0x80) != 0 {
            self.ack = ERR_INVALID_MSG_ID;
        }

        self.ack = self.handle_payload(&data[std::mem::size_of::<Header>()..]);
        if self.ack != ERR_NONE {
            return;
        }

        if header.slot_2_offset != 0 {
            self.ack = self.handle_payload(
                &data[std::mem::size_of::<Header>() + header.slot_2_offset as usize..],
            );
            if self.ack != ERR_NONE {
                return;
            }
        }

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_CTL_REG,
            self.fpga_flags_internal,
        );

        self.ack = header.msg_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    #[test]
    fn cpu_idx() {
        let mut rng = rand::thread_rng();
        let idx = rng.gen();
        let cpu = CPUEmulator::new(idx, 249);
        assert_eq!(cpu.idx(), idx);
    }

    #[test]
    fn num_transducers() {
        let cpu = CPUEmulator::new(0, 249);
        assert_eq!(cpu.num_transducers(), 249);
    }
}
