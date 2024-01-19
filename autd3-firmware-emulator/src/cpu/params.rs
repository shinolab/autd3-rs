/*
 * File: params.rs
 * Project: cpu
 * Created Date: 07/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 19/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

pub const CPU_VERSION_MAJOR: u16 = 0x8E;
pub const CPU_VERSION_MINOR: u16 = 0x01;

pub const BRAM_SELECT_CONTROLLER: u8 = 0x0;
pub const BRAM_SELECT_MOD: u8 = 0x1;
pub const BRAM_SELECT_NORMAL: u8 = 0x2;
pub const BRAM_SELECT_STM: u8 = 0x3;

pub const CTL_FLAG_OP_MODE: u16 = 1 << 9;
pub const CTL_REG_STM_GAIN_MODE: u16 = 1 << 10;
pub const CTL_FLAG_USE_STM_FINISH_IDX: u16 = 1 << 11;
pub const CTL_FLAG_USE_STM_START_IDX: u16 = 1 << 12;
pub const CTL_FLAG_FORCE_FAN: u16 = 1 << 13;

pub const READS_FPGA_STATE_ENABLED_BIT: u8 = 7;
pub const READS_FPGA_STATE_ENABLED: u8 = 1 << READS_FPGA_STATE_ENABLED_BIT;

pub const BRAM_ADDR_CTL_REG: u16 = 0x000;
pub const BRAM_ADDR_FPGA_STATE: u16 = 0x001;
pub const BRAM_ADDR_MOD_ADDR_OFFSET: u16 = 0x020;
pub const BRAM_ADDR_MOD_CYCLE: u16 = 0x021;
pub const BRAM_ADDR_MOD_FREQ_DIV_0: u16 = 0x022;
pub const BRAM_ADDR_VERSION_NUM: u16 = 0x030;
pub const BRAM_ADDR_VERSION_NUM_MINOR: u16 = 0x031;
pub const BRAM_ADDR_SILENCER_UPDATE_RATE_INTENSITY: u16 = 0x040;
pub const BRAM_ADDR_SILENCER_UPDATE_RATE_PHASE: u16 = 0x041;
pub const BRAM_ADDR_SILENCER_CTL_FLAG: u16 = 0x042;
pub const BRAM_ADDR_SILENCER_COMPLETION_STEPS_INTENSITY: u16 = 0x043;
pub const BRAM_ADDR_SILENCER_COMPLETION_STEPS_PHASE: u16 = 0x044;
pub const BRAM_ADDR_STM_ADDR_OFFSET: u16 = 0x050;
pub const BRAM_ADDR_STM_CYCLE: u16 = 0x051;
pub const BRAM_ADDR_STM_FREQ_DIV_0: u16 = 0x052;
pub const BRAM_ADDR_SOUND_SPEED_0: u16 = 0x054;
pub const BRAM_ADDR_STM_START_IDX: u16 = 0x056;
pub const BRAM_ADDR_STM_FINISH_IDX: u16 = 0x057;
pub const BRAM_ADDR_DEBUG_OUT_IDX: u16 = 0x0F0;
pub const BRAM_ADDR_MOD_DELAY_BASE: u16 = 0x200;

pub const ERR_NONE: u8 = 0x00;
pub const ERR_NOT_SUPPORTED_TAG: u8 = 0x80;
pub const ERR_INVALID_MSG_ID: u8 = 0x81;
pub const ERR_FREQ_DIV_TOO_SMALL: u8 = 0x82;
pub const ERR_COMPLETION_STEPS_TOO_LARGE: u8 = 0x83;
