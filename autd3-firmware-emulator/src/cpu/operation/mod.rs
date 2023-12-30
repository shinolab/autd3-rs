/*
 * File: mod.rs
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

pub const TAG_CLEAR: u8 = 0x01;
pub const TAG_SYNC: u8 = 0x02;
pub const TAG_FIRM_INFO: u8 = 0x03;
pub const TAG_MODULATION: u8 = 0x10;
pub const TAG_MODULATION_DELAY: u8 = 0x11;
pub const TAG_SILENCER: u8 = 0x20;
pub const TAG_GAIN: u8 = 0x30;
pub const TAG_FOCUS_STM: u8 = 0x40;
pub const TAG_GAIN_STM: u8 = 0x50;
pub const TAG_FORCE_FAN: u8 = 0x60;
pub const TAG_READS_FPGA_INFO: u8 = 0x61;
pub const TAG_DEBUG: u8 = 0xF0;

mod clear;
mod debug;
mod force_fan;
mod gain;
mod info;
mod mod_delay;
mod modulation;
mod reads_fpga_info;
mod silecer;
mod stm;
mod sync;
