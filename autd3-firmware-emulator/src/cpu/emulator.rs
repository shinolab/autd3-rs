/*
 * File: emulator.rs
 * Project: src
 * Created Date: 06/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 06/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::cpu::{Header, TxDatagram};

use crate::fpga::emulator::FPGAEmulator;

use super::params::*;

pub struct CPUEmulator {
    idx: usize,
    ack: u8,
    rx_data: u8,
    read_fpga_info: bool,
    read_fpga_info_store: bool,
    mod_cycle: u32,
    stm_cycle: u32,
    gain_stm_mode: u16,
    fpga: FPGAEmulator,
    synchronized: bool,
    num_transducers: usize,
    fpga_flags_internal: u16,
}

impl CPUEmulator {
    pub fn new(id: usize, num_transducers: usize) -> Self {
        let mut s = Self {
            idx: id,
            ack: 0x00,
            rx_data: 0x00,
            read_fpga_info: false,
            read_fpga_info_store: false,
            mod_cycle: 0,
            stm_cycle: 0,
            gain_stm_mode: 0,
            fpga: FPGAEmulator::new(num_transducers),
            synchronized: false,
            num_transducers,
            fpga_flags_internal: 0x0000,
        };
        s.init();
        s
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn num_transducers(&self) -> usize {
        self.num_transducers
    }

    pub fn ack(&self) -> u8 {
        self.ack
    }

    pub fn rx_data(&self) -> u8 {
        self.rx_data
    }

    pub fn fpga(&self) -> &FPGAEmulator {
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
        self.clear();
    }

    pub fn update(&mut self) {
        if self.should_update() {
            self.rx_data = self.read_fpga_info() as _;
        }
    }

    pub fn should_update(&self) -> bool {
        self.read_fpga_info
    }
}

impl CPUEmulator {
    fn get_addr(select: u8, addr: u16) -> u16 {
        ((select as u16 & 0x0003) << 14) | (addr & 0x3FFF)
    }

    fn bram_read(&self, select: u8, addr: u16) -> u16 {
        let addr = Self::get_addr(select, addr);
        self.fpga.read(addr)
    }

    fn bram_write(&mut self, select: u8, addr: u16, data: u16) {
        let addr = Self::get_addr(select, addr);
        self.fpga.write(addr, data)
    }

    fn bram_cpy(&mut self, select: u8, addr_base: u16, data: *const u16, size: usize) {
        let mut addr = Self::get_addr(select, addr_base);
        let mut src = data;
        (0..size).for_each(|_| unsafe {
            self.fpga.write(addr, src.read());
            addr += 1;
            src = src.add(1);
        })
    }

    fn bram_set(&mut self, select: u8, addr_base: u16, value: u16, size: usize) {
        let mut addr = Self::get_addr(select, addr_base);
        (0..size).for_each(|_| {
            self.fpga.write(addr, value);
            addr += 1;
        })
    }

    fn synchronize(&mut self) {
        self.synchronized = true;

        // Do nothing to sync
    }

    fn write_mod(&mut self, data: &[u8]) {
        let flag = data[1];

        let write = ((data[3] as u16) << 8) | data[2] as u16;
        let data = if (flag & MODULATION_FLAG_BEGIN) == MODULATION_FLAG_BEGIN {
            self.mod_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_ADDR_OFFSET, 0);
            let freq_div = ((data[7] as u32) << 24)
                | ((data[6] as u32) << 16)
                | ((data[5] as u32) << 8)
                | data[4] as u32;
            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );
            data[8..].as_ptr() as *const u16
        } else {
            data[4..].as_ptr() as *const u16
        };

        let page_capacity =
            (self.mod_cycle & !MOD_BUF_PAGE_SIZE_MASK) + MOD_BUF_PAGE_SIZE - self.mod_cycle;

        if write as u32 <= page_capacity {
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_PAGE_SIZE_MASK) >> 1) as u16,
                data,
                ((write + 1) >> 1) as usize,
            );
            self.mod_cycle += write as u32;
        } else {
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_PAGE_SIZE_MASK) >> 1) as u16,
                data,
                (page_capacity >> 1) as usize,
            );
            self.mod_cycle += page_capacity;
            let data = unsafe { data.add(page_capacity as _) };
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_ADDR_OFFSET,
                ((self.mod_cycle & !MOD_BUF_PAGE_SIZE_MASK) >> MOD_BUF_PAGE_SIZE_WIDTH) as u16,
            );
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_PAGE_SIZE_MASK) >> 1) as _,
                data,
                ((write as u32 - page_capacity + 1) >> 1) as _,
            );
            self.mod_cycle += write as u32 - page_capacity;
        }

        if (flag & MODULATION_FLAG_END) == MODULATION_FLAG_END {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_CYCLE,
                (self.mod_cycle.max(1) - 1) as _,
            );
        }
    }

    fn config_silencer(&mut self, data: &[u8]) {
        let step_intensity = ((data[3] as u16) << 8) | data[2] as u16;
        let step_phase = ((data[5] as u16) << 8) | data[4] as u16;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_SILENT_STEP_INTENSITY,
            step_intensity,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_SILENT_STEP_PHASE,
            step_phase,
        );
    }

    fn config_debug(&mut self, data: &[u8]) {
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_DEBUG_OUT_IDX,
            data[0] as u16,
        );
    }

    fn write_mod_delay(&mut self, data: &[u8]) {
        let delays = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u16, self.num_transducers)
        };
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_DELAY_BASE,
            delays.as_ptr(),
            delays.len(),
        );
    }

    fn write_gain(&mut self, data: &[u8]) {
        self.fpga_flags_internal &= !CTL_FLAG_OP_MODE;

        let data = unsafe {
            std::slice::from_raw_parts(data[2..].as_ptr() as *const u16, (data.len() - 2) >> 1)
        };

        (0..self.num_transducers)
            .for_each(|i| self.bram_write(BRAM_SELECT_NORMAL, i as _, data[i]));
    }

    fn write_focus_stm(&mut self, data: &[u8]) {
        let flag = data[1];

        let size = (data[3] as u32) << 8 | data[2] as u32;

        let mut src = if (flag & FOCUS_STM_FLAG_BEGIN) == FOCUS_STM_FLAG_BEGIN {
            self.stm_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_ADDR_OFFSET, 0);
            let freq_div = ((data[7] as u32) << 24)
                | ((data[6] as u32) << 16)
                | ((data[5] as u32) << 8)
                | data[4] as u32;
            let sound_speed = ((data[11] as u32) << 24)
                | ((data[10] as u32) << 16)
                | ((data[9] as u32) << 8)
                | data[8] as u32;
            let start_idx = ((data[13] as u16) << 8) | data[12] as u16;
            let finish_idx = ((data[15] as u16) << 8) | data[14] as u16;

            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );
            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_SOUND_SPEED_0,
                &sound_speed as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );

            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_START_IDX, start_idx);
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FINISH_IDX, finish_idx);

            if (flag & FOCUS_STM_FLAG_USE_START_IDX) == FOCUS_STM_FLAG_USE_START_IDX {
                self.fpga_flags_internal |= CTL_FLAG_USE_STM_START_IDX;
            } else {
                self.fpga_flags_internal &= !CTL_FLAG_USE_STM_START_IDX;
            }
            if (flag & FOCUS_STM_FLAG_USE_FINISH_IDX) == FOCUS_STM_FLAG_USE_FINISH_IDX {
                self.fpga_flags_internal |= CTL_FLAG_USE_STM_FINISH_IDX;
            } else {
                self.fpga_flags_internal &= !CTL_FLAG_USE_STM_FINISH_IDX;
            }

            unsafe { data.as_ptr().add(16) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(4) as *const u16 }
        };

        let page_capacity = (self.stm_cycle & !POINT_STM_BUF_PAGE_SIZE_MASK)
            + POINT_STM_BUF_PAGE_SIZE
            - self.stm_cycle;
        if size <= page_capacity {
            let mut dst = ((self.stm_cycle & POINT_STM_BUF_PAGE_SIZE_MASK) << 3) as u16;
            (0..size as usize).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                dst += 4;
            });
            self.stm_cycle += size;
        } else {
            let mut dst = ((self.stm_cycle & POINT_STM_BUF_PAGE_SIZE_MASK) << 3) as u16;
            (0..page_capacity as usize).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                dst += 4;
            });
            self.stm_cycle += page_capacity;

            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_ADDR_OFFSET,
                ((self.stm_cycle & !POINT_STM_BUF_PAGE_SIZE_MASK) >> POINT_STM_BUF_PAGE_SIZE_WIDTH)
                    as _,
            );

            let mut dst = ((self.stm_cycle & POINT_STM_BUF_PAGE_SIZE_MASK) << 3) as u16;
            let cnt = size - page_capacity;
            (0..cnt as usize).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
                dst += 4;
            });
            self.stm_cycle += size - page_capacity;
        }

        if (flag & FOCUS_STM_FLAG_END) == FOCUS_STM_FLAG_END {
            self.fpga_flags_internal |= CTL_FLAG_OP_MODE;
            self.fpga_flags_internal &= !CTL_REG_STM_GAIN_MODE;
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_CYCLE,
                (self.stm_cycle.max(1) - 1) as _,
            );
        }
    }

    fn write_gain_stm(&mut self, data: &[u8]) {
        let flag = data[1];

        let send = (flag >> 6) + 1;
        let src_base = if (flag & GAIN_STM_FLAG_BEGIN) == GAIN_STM_FLAG_BEGIN {
            self.stm_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_ADDR_OFFSET, 0);

            self.gain_stm_mode = (data[3] as u16) << 8 | data[2] as u16;

            let freq_div = ((data[7] as u32) << 24)
                | ((data[6] as u32) << 16)
                | ((data[5] as u32) << 8)
                | data[4] as u32;
            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );

            let start_idx = ((data[9] as u16) << 8) | data[8] as u16;
            let finish_idx = ((data[11] as u16) << 8) | data[10] as u16;

            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_START_IDX, start_idx);
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FINISH_IDX, finish_idx);

            if (flag & GAIN_STM_FLAG_USE_START_IDX) == GAIN_STM_FLAG_USE_START_IDX {
                self.fpga_flags_internal |= CTL_FLAG_USE_STM_START_IDX;
            } else {
                self.fpga_flags_internal &= !CTL_FLAG_USE_STM_START_IDX;
            }
            if (flag & GAIN_STM_FLAG_USE_FINISH_IDX) == GAIN_STM_FLAG_USE_FINISH_IDX {
                self.fpga_flags_internal |= CTL_FLAG_USE_STM_FINISH_IDX;
            } else {
                self.fpga_flags_internal &= !CTL_FLAG_USE_STM_FINISH_IDX;
            }

            unsafe { data.as_ptr().add(12) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(2) as *const u16 }
        };

        let mut src = src_base;
        let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;

        if self.gain_stm_mode == GAIN_STM_MODE_INTENSITY_PHASE_FULL {
            self.stm_cycle += 1;
            (0..self.num_transducers).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
            });
        } else if self.gain_stm_mode == GAIN_STM_MODE_PHASE_FULL {
            (0..self.num_transducers).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (src.read() & 0x00FF));
                dst += 1;
                src = src.add(1);
            });
            self.stm_cycle += 1;

            if send > 1 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | ((src.read() >> 8) & 0x00FF));
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle += 1;
            }
        } else if self.gain_stm_mode == GAIN_STM_MODE_PHASE_HALF {
            (0..self.num_transducers).for_each(|_| unsafe {
                let phase = src.read() & 0x000F;
                self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                dst += 1;
                src = src.add(1);
            });
            self.stm_cycle += 1;

            if send > 1 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 4) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle += 1;
            }

            if send > 2 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 8) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle += 1;
            }

            if send > 3 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 12) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle += 1;
            }
        }

        if self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK == 0 {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_ADDR_OFFSET,
                ((self.stm_cycle & !GAIN_STM_BUF_PAGE_SIZE_MASK) >> GAIN_STM_BUF_PAGE_SIZE_WIDTH)
                    as _,
            );
        }

        if (flag & GAIN_STM_FLAG_END) == GAIN_STM_FLAG_END {
            self.fpga_flags_internal |= CTL_FLAG_OP_MODE;
            self.fpga_flags_internal |= CTL_REG_STM_GAIN_MODE;
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_CYCLE,
                (self.stm_cycle.max(1) - 1) as _,
            );
        }
    }

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

    fn read_fpga_info(&self) -> u16 {
        self.bram_read(BRAM_SELECT_CONTROLLER, BRAM_ADDR_FPGA_INFO)
    }

    fn configure_force_fan(&mut self, data: &[u8]) {
        if data[0] != 0x00 {
            self.fpga_flags_internal |= CTL_FLAG_FORCE_FAN;
        } else {
            self.fpga_flags_internal &= !CTL_FLAG_FORCE_FAN;
        }
    }

    fn configure_reads_fpga_info(&mut self, data: &[u8]) {
        self.read_fpga_info = data[0] != 0x00;
    }

    fn clear(&mut self) {
        let freq_div_4k = 5120;

        self.read_fpga_info = false;

        self.fpga_flags_internal = 0x0000;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_CTL_REG,
            self.fpga_flags_internal,
        );

        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_SILENT_STEP_INTENSITY, 256);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_SILENT_STEP_PHASE, 256);

        self.stm_cycle = 0;

        self.mod_cycle = 2;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_CYCLE,
            (self.mod_cycle.max(1) - 1) as _,
        );
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_FREQ_DIV_0,
            &freq_div_4k as *const _ as _,
            std::mem::size_of::<u32>() >> 1,
        );
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_ADDR_OFFSET, 0x0000);
        self.bram_write(BRAM_SELECT_MOD, 0, 0xFFFF);

        self.bram_set(BRAM_SELECT_NORMAL, 0, 0x0000, self.num_transducers << 1);

        self.bram_set(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_DELAY_BASE,
            0x0000,
            self.num_transducers,
        );

        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_DEBUG_OUT_IDX, 0xFF);
    }

    fn handle_payload(&mut self, tag: u8, data: &[u8]) {
        match tag {
            TAG_NONE => {}
            TAG_CLEAR => self.clear(),
            TAG_SYNC => self.synchronize(),
            TAG_FIRM_INFO => match data[1] {
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
                    unimplemented!("Unsupported firmware info type")
                }
            },
            TAG_UPDATE_FLAGS => (),
            TAG_MODULATION => self.write_mod(data),
            TAG_MODULATION_DELAY => self.write_mod_delay(&data[2..]),
            TAG_SILENCER => self.config_silencer(data),
            TAG_GAIN => self.write_gain(data),
            TAG_FOCUS_STM => self.write_focus_stm(data),
            TAG_GAIN_STM => self.write_gain_stm(data),
            TAG_FORCE_FAN => self.configure_force_fan(&data[2..]),
            TAG_READS_FPGA_INFO => self.configure_reads_fpga_info(&data[2..]),
            TAG_DEBUG => self.config_debug(&data[2..]),
            _ => {
                unimplemented!("Unsupported tag")
            }
        }
    }

    fn ecat_recv(&mut self, data: &[u8]) {
        let header = unsafe { &*(data.as_ptr() as *const Header) };

        if self.ack == header.msg_id {
            return;
        }

        self.handle_payload(
            data[std::mem::size_of::<Header>()],
            &data[std::mem::size_of::<Header>()..],
        );

        if header.slot_2_offset != 0 {
            self.handle_payload(
                data[std::mem::size_of::<Header>() + header.slot_2_offset as usize],
                &data[std::mem::size_of::<Header>() + header.slot_2_offset as usize..],
            )
        }

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_CTL_REG,
            self.fpga_flags_internal,
        );

        if self.read_fpga_info {
            self.rx_data = self.read_fpga_info() as _;
        }
        self.ack = header.msg_id;
    }
}
