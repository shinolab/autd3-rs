/*
 * File: fpga_emulator.rs
 * Project: src
 * Created Date: 06/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 28/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::defined::{float, PI};

use crate::error::AUTDFirmwareEmulatorError;

use super::params::*;

use chrono::{Local, LocalResult, TimeZone, Utc};
use num_integer::Roots;

pub struct FPGAEmulator {
    controller_bram: Vec<u16>,
    modulator_bram: Vec<u16>,
    normal_op_bram: Vec<u16>,
    stm_op_bram: Vec<u16>,
    num_transducers: usize,
    tr_pos: Vec<u64>,
}

impl FPGAEmulator {
    pub(crate) fn new(num_transducers: usize) -> Self {
        Self {
            controller_bram: vec![0x0000; 1280],
            modulator_bram: vec![0x0000; 32768],
            normal_op_bram: vec![0x0000; 512],
            stm_op_bram: vec![0x0000; 524288],
            num_transducers,
            tr_pos: vec![
                0x00000000, 0x01960000, 0x032c0000, 0x04c30000, 0x06590000, 0x07ef0000, 0x09860000,
                0x0b1c0000, 0x0cb30000, 0x0e490000, 0x0fdf0000, 0x11760000, 0x130c0000, 0x14a30000,
                0x16390000, 0x17d00000, 0x19660000, 0x1afc0000, 0x00000196, 0x04c30196, 0x06590196,
                0x07ef0196, 0x09860196, 0x0b1c0196, 0x0cb30196, 0x0e490196, 0x0fdf0196, 0x11760196,
                0x130c0196, 0x14a30196, 0x16390196, 0x17d00196, 0x1afc0196, 0x0000032c, 0x0196032c,
                0x032c032c, 0x04c3032c, 0x0659032c, 0x07ef032c, 0x0986032c, 0x0b1c032c, 0x0cb3032c,
                0x0e49032c, 0x0fdf032c, 0x1176032c, 0x130c032c, 0x14a3032c, 0x1639032c, 0x17d0032c,
                0x1966032c, 0x1afc032c, 0x000004c3, 0x019604c3, 0x032c04c3, 0x04c304c3, 0x065904c3,
                0x07ef04c3, 0x098604c3, 0x0b1c04c3, 0x0cb304c3, 0x0e4904c3, 0x0fdf04c3, 0x117604c3,
                0x130c04c3, 0x14a304c3, 0x163904c3, 0x17d004c3, 0x196604c3, 0x1afc04c3, 0x00000659,
                0x01960659, 0x032c0659, 0x04c30659, 0x06590659, 0x07ef0659, 0x09860659, 0x0b1c0659,
                0x0cb30659, 0x0e490659, 0x0fdf0659, 0x11760659, 0x130c0659, 0x14a30659, 0x16390659,
                0x17d00659, 0x19660659, 0x1afc0659, 0x000007ef, 0x019607ef, 0x032c07ef, 0x04c307ef,
                0x065907ef, 0x07ef07ef, 0x098607ef, 0x0b1c07ef, 0x0cb307ef, 0x0e4907ef, 0x0fdf07ef,
                0x117607ef, 0x130c07ef, 0x14a307ef, 0x163907ef, 0x17d007ef, 0x196607ef, 0x1afc07ef,
                0x00000986, 0x01960986, 0x032c0986, 0x04c30986, 0x06590986, 0x07ef0986, 0x09860986,
                0x0b1c0986, 0x0cb30986, 0x0e490986, 0x0fdf0986, 0x11760986, 0x130c0986, 0x14a30986,
                0x16390986, 0x17d00986, 0x19660986, 0x1afc0986, 0x00000b1c, 0x01960b1c, 0x032c0b1c,
                0x04c30b1c, 0x06590b1c, 0x07ef0b1c, 0x09860b1c, 0x0b1c0b1c, 0x0cb30b1c, 0x0e490b1c,
                0x0fdf0b1c, 0x11760b1c, 0x130c0b1c, 0x14a30b1c, 0x16390b1c, 0x17d00b1c, 0x19660b1c,
                0x1afc0b1c, 0x00000cb3, 0x01960cb3, 0x032c0cb3, 0x04c30cb3, 0x06590cb3, 0x07ef0cb3,
                0x09860cb3, 0x0b1c0cb3, 0x0cb30cb3, 0x0e490cb3, 0x0fdf0cb3, 0x11760cb3, 0x130c0cb3,
                0x14a30cb3, 0x16390cb3, 0x17d00cb3, 0x19660cb3, 0x1afc0cb3, 0x00000e49, 0x01960e49,
                0x032c0e49, 0x04c30e49, 0x06590e49, 0x07ef0e49, 0x09860e49, 0x0b1c0e49, 0x0cb30e49,
                0x0e490e49, 0x0fdf0e49, 0x11760e49, 0x130c0e49, 0x14a30e49, 0x16390e49, 0x17d00e49,
                0x19660e49, 0x1afc0e49, 0x00000fdf, 0x01960fdf, 0x032c0fdf, 0x04c30fdf, 0x06590fdf,
                0x07ef0fdf, 0x09860fdf, 0x0b1c0fdf, 0x0cb30fdf, 0x0e490fdf, 0x0fdf0fdf, 0x11760fdf,
                0x130c0fdf, 0x14a30fdf, 0x16390fdf, 0x17d00fdf, 0x19660fdf, 0x1afc0fdf, 0x00001176,
                0x01961176, 0x032c1176, 0x04c31176, 0x06591176, 0x07ef1176, 0x09861176, 0x0b1c1176,
                0x0cb31176, 0x0e491176, 0x0fdf1176, 0x11761176, 0x130c1176, 0x14a31176, 0x16391176,
                0x17d01176, 0x19661176, 0x1afc1176, 0x0000130c, 0x0196130c, 0x032c130c, 0x04c3130c,
                0x0659130c, 0x07ef130c, 0x0986130c, 0x0b1c130c, 0x0cb3130c, 0x0e49130c, 0x0fdf130c,
                0x1176130c, 0x130c130c, 0x14a3130c, 0x1639130c, 0x17d0130c, 0x1966130c, 0x1afc130c,
                0x000014a3, 0x019614a3, 0x032c14a3, 0x04c314a3, 0x065914a3, 0x07ef14a3, 0x098614a3,
                0x0b1c14a3, 0x0cb314a3, 0x0e4914a3, 0x0fdf14a3, 0x117614a3, 0x130c14a3, 0x14a314a3,
                0x163914a3, 0x17d014a3, 0x196614a3, 0x1afc14a3,
            ],
        }
    }

    pub(crate) fn init(&mut self) {
        self.controller_bram[ADDR_VERSION_NUM] =
            (ENABLED_FEATURES_BITS as u16) << 8 | VERSION_NUM_MAJOR as u16;
        self.controller_bram[ADDR_VERSION_NUM_MINOR] = VERSION_NUM_MINOR as u16;
    }

    pub(crate) fn read(&self, addr: u16) -> u16 {
        let select = (addr >> 14) & 0x0003;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => self.controller_bram[addr],
            _ => unreachable!(),
        }
    }

    pub(crate) fn write(&mut self, addr: u16, data: u16) {
        let select = (addr >> 14) & 0x0003;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => self.controller_bram[addr] = data,
            BRAM_SELECT_MOD => {
                let offset = self.controller_bram[ADDR_MOD_ADDR_OFFSET];
                let addr = (offset as usize) << 14 | addr;
                self.modulator_bram[addr] = data;
            }
            BRAM_SELECT_NORMAL => self.normal_op_bram[addr] = data,
            BRAM_SELECT_STM => {
                let offset = self.controller_bram[ADDR_STM_ADDR_OFFSET];
                let addr = (offset as usize) << 14 | addr;
                self.stm_op_bram[addr] = data
            }
            _ => unreachable!(),
        }
    }

    pub fn assert_thermal_sensor(&mut self) {
        self.controller_bram[ADDR_FPGA_INFO] |= 0x0001;
    }

    pub fn deassert_thermal_sensor(&mut self) {
        self.controller_bram[ADDR_FPGA_INFO] &= !0x0001;
    }

    pub fn is_force_fan(&self) -> bool {
        (self.controller_bram[ADDR_CTL_REG] & (1 << CTL_REG_FORCE_FAN_BIT)) != 0
            || (self.controller_bram[ADDR_CTL_REG] & (1 << CTL_REG_FORCE_FAN_EX_BIT)) != 0
    }

    pub fn is_stm_mode(&self) -> bool {
        (self.controller_bram[ADDR_CTL_REG] & (1 << CTL_REG_OP_MODE_BIT)) != 0
    }

    pub fn is_stm_gain_mode(&self) -> bool {
        (self.controller_bram[ADDR_CTL_REG] & (1 << CTL_REG_STM_GAIN_MODE_BIT)) != 0
    }

    pub fn silencer_update_rate_intensity(&self) -> u16 {
        self.controller_bram[ADDR_SILENCER_UPDATE_RATE_INTENSITY]
    }

    pub fn silencer_update_rate_phase(&self) -> u16 {
        self.controller_bram[ADDR_SILENCER_UPDATE_RATE_PHASE]
    }

    pub fn silencer_completion_steps_intensity(&self) -> u16 {
        self.controller_bram[ADDR_SILENCER_COMPLETION_STEPS_INTENSITY]
    }

    pub fn silencer_completion_steps_phase(&self) -> u16 {
        self.controller_bram[ADDR_SILENCER_COMPLETION_STEPS_PHASE]
    }

    pub fn silencer_fixed_completion_steps_mode(&self) -> bool {
        self.controller_bram[ADDR_SILENCER_CTL_FLAG]
            & (1 << SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS_BIT)
            != 0
    }

    pub fn mod_delays(&self) -> Vec<u16> {
        self.controller_bram[ADDR_MOD_DELAY_BASE..]
            .iter()
            .take(self.num_transducers)
            .copied()
            .collect()
    }

    pub fn stm_frequency_division(&self) -> u32 {
        ((self.controller_bram[ADDR_STM_FREQ_DIV_1] as u32) << 16) & 0xFFFF0000
            | self.controller_bram[ADDR_STM_FREQ_DIV_0] as u32 & 0x0000FFFF
    }

    pub fn stm_cycle(&self) -> usize {
        self.controller_bram[ADDR_STM_CYCLE] as usize + 1
    }

    pub fn sound_speed(&self) -> u32 {
        ((self.controller_bram[ADDR_SOUND_SPEED_1] as u32) << 16) & 0xFFFF0000
            | self.controller_bram[ADDR_SOUND_SPEED_0] as u32 & 0x0000FFFF
    }

    pub fn stm_start_idx(&self) -> Option<u16> {
        if self.controller_bram[ADDR_CTL_REG] & (1 << CTL_FLAG_USE_STM_START_IDX_BIT) != 0 {
            Some(self.controller_bram[ADDR_STM_START_IDX])
        } else {
            None
        }
    }

    pub fn stm_finish_idx(&self) -> Option<u16> {
        if self.controller_bram[ADDR_CTL_REG] & (1 << CTL_FLAG_USE_STM_FINISH_IDX_BIT) != 0 {
            Some(self.controller_bram[ADDR_STM_FINISH_IDX])
        } else {
            None
        }
    }

    pub fn modulation_frequency_division(&self) -> u32 {
        ((self.controller_bram[ADDR_MOD_FREQ_DIV_1] as u32) << 16) & 0xFFFF0000
            | self.controller_bram[ADDR_MOD_FREQ_DIV_0] as u32 & 0x0000FFFF
    }

    pub fn modulation_cycle(&self) -> usize {
        self.controller_bram[ADDR_MOD_CYCLE] as usize + 1
    }

    pub fn modulation_at(&self, idx: usize) -> u8 {
        let m = if idx % 2 == 0 {
            self.modulator_bram[idx >> 1] & 0xFF
        } else {
            self.modulator_bram[idx >> 1] >> 8
        };
        m as u8
    }

    pub fn modulation(&self) -> Vec<u8> {
        let cycle = self.modulation_cycle();
        let mut m = Vec::with_capacity(cycle);
        (0..cycle >> 1).for_each(|i| {
            let b = self.modulator_bram[i];
            m.push((b & 0x00FF) as u8);
            m.push(((b >> 8) & 0x00FF) as u8);
        });
        if cycle % 2 != 0 {
            let b = self.modulator_bram[cycle >> 1];
            m.push((b & 0x00FF) as u8);
        }
        m
    }

    pub fn is_outputting(&self) -> bool {
        if self.modulation().iter().all(|&m| m == 0) {
            return false;
        }
        if !self.is_stm_mode() {
            return self.intensities_and_phases(0).iter().any(|&d| d.0 != 0);
        }
        true
    }

    pub fn intensities_and_phases(&self, idx: usize) -> Vec<(u8, u8)> {
        if self.is_stm_mode() {
            if self.is_stm_gain_mode() {
                self.gain_stm_intensities_and_phases(idx)
            } else {
                self.focus_stm_intensities_and_phases(idx)
            }
        } else {
            self.normal_intensities_and_phases()
        }
    }

    pub fn debug_output_idx(&self) -> Option<u8> {
        let idx = self.controller_bram[BRAM_ADDR_DEBUG_OUT_IDX];
        if idx == 0xFF {
            None
        } else {
            Some(idx as u8)
        }
    }

    fn normal_intensities_and_phases(&self) -> Vec<(u8, u8)> {
        self.normal_op_bram
            .iter()
            .take(self.num_transducers)
            .map(|d| {
                let intensity = (d >> 8) & 0xFF;
                let phase = d & 0xFF;
                (intensity as u8, phase as u8)
            })
            .collect()
    }

    fn gain_stm_intensities_and_phases(&self, idx: usize) -> Vec<(u8, u8)> {
        self.stm_op_bram
            .iter()
            .skip(256 * idx)
            .take(self.num_transducers)
            .map(|&d| {
                let intensity = (d >> 8) & 0xFF;
                let phase = d & 0xFF;
                (intensity as u8, phase as u8)
            })
            .collect()
    }

    fn focus_stm_intensities_and_phases(&self, idx: usize) -> Vec<(u8, u8)> {
        let sound_speed = self.sound_speed() as u64;
        let intensity = self.stm_op_bram[8 * idx + 3] >> 6 & 0x00FF;

        let mut x = (self.stm_op_bram[8 * idx + 1] as u32) << 16 & 0x30000;
        x |= self.stm_op_bram[8 * idx] as u32;
        let x = if (x & 0x20000) != 0 {
            (x | 0xFFFC0000) as i32
        } else {
            x as i32
        };
        let mut y = (self.stm_op_bram[8 * idx + 2] as u32) << 14 & 0x3C000;
        y |= self.stm_op_bram[8 * idx + 1] as u32 >> 2;
        let y = if (y & 0x20000) != 0 {
            (y | 0xFFFC0000) as i32
        } else {
            y as i32
        };
        let mut z = (self.stm_op_bram[8 * idx + 3] as u32) << 12 & 0x3F000;
        z |= self.stm_op_bram[8 * idx + 2] as u32 >> 4;
        let z = if (z & 0x20000) != 0 {
            (z | 0xFFFC0000) as i32
        } else {
            z as i32
        };
        self.tr_pos
            .iter()
            .map(|&tr| {
                let tr_z = ((tr >> 32) & 0xFFFF) as i16 as i32;
                let tr_x = ((tr >> 16) & 0xFFFF) as i16 as i32;
                let tr_y = (tr & 0xFFFF) as i16 as i32;
                let d2 =
                    (x - tr_x) * (x - tr_x) + (y - tr_y) * (y - tr_y) + (z - tr_z) * (z - tr_z);
                let dist = d2.sqrt() as u64;
                let q = (dist << 18) / sound_speed;
                let p = q & 0xFF;
                (intensity as u8, p as u8)
            })
            .collect()
    }

    pub fn to_pulse_width(a: u8, b: u8) -> u16 {
        let a = a as float / 255.0;
        let b = b as float / 255.0;
        ((a * b).asin() / PI * 512.0).round() as u16
    }

    pub fn local_tr_pos(&self) -> &[u64] {
        &self.tr_pos
    }

    pub fn ec_time_now() -> u64 {
        (Local::now()
            .signed_duration_since(Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).single().unwrap()))
        .num_nanoseconds()
        .unwrap() as _
    }

    pub fn ec_time_with_utc_ymdhms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> Result<u64, AUTDFirmwareEmulatorError> {
        match Utc.with_ymd_and_hms(year, month, day, hour, min, sec) {
            LocalResult::Single(date) => match (date
                .signed_duration_since(Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).single().unwrap()))
            .num_nanoseconds()
            {
                Some(n) if n < 0 => Err(AUTDFirmwareEmulatorError::InvalidDateTime),
                Some(n) => Ok(n as _),
                None => Err(AUTDFirmwareEmulatorError::InvalidDateTime),
            },
            _ => Err(AUTDFirmwareEmulatorError::InvalidDateTime),
        }
    }

    pub const fn systime(ec_time: u64) -> u64 {
        ((ec_time as u128 * autd3_driver::fpga::FPGA_CLK_FREQ as u128) / 1000000000) as _
    }

    pub fn stm_idx_from_systime(&self, systime: u64) -> usize {
        (systime / self.stm_frequency_division() as u64) as usize % self.stm_cycle()
    }

    pub fn mod_idx_from_systime(&self, systime: u64) -> usize {
        (systime / self.modulation_frequency_division() as u64) as usize % self.modulation_cycle()
    }
}

#[cfg(test)]

mod tests {
    use crate::cpu::params::{
        BRAM_ADDR_MOD_CYCLE, BRAM_ADDR_MOD_FREQ_DIV_0, BRAM_ADDR_STM_CYCLE,
        BRAM_ADDR_STM_FREQ_DIV_0,
    };

    use super::*;

    static ASIN_TABLE: &[u8; 65536] = include_bytes!("asin.dat");

    fn to_pulse_width_actual(a: u8, b: u8) -> u16 {
        let r = ASIN_TABLE[a as usize * b as usize];
        let full_width = a == 0xFF && b == 0xFF;
        if full_width {
            r as u16 | 0x0100
        } else {
            r as u16
        }
    }

    #[test]
    fn test_to_pulse_width() {
        for a in 0x00..=0xFF {
            for b in 0x00..=0xFF {
                assert_eq!(
                    to_pulse_width_actual(a, b),
                    FPGAEmulator::to_pulse_width(a, b)
                );
            }
        }
    }

    #[test]
    #[should_panic]
    fn read_panic() {
        let fpga = FPGAEmulator::new(249);
        let addr = BRAM_SELECT_MOD << 14;
        fpga.read(addr);
    }

    #[test]
    fn modulation() {
        let mut fpga = FPGAEmulator::new(249);
        fpga.modulator_bram[0] = 0x1234;
        fpga.modulator_bram[1] = 0x5678;
        fpga.controller_bram[ADDR_MOD_CYCLE] = 3 - 1;
        assert_eq!(3, fpga.modulation_cycle());
        assert_eq!(0x34, fpga.modulation_at(0));
        assert_eq!(0x12, fpga.modulation_at(1));
        assert_eq!(0x78, fpga.modulation_at(2));
        let m = fpga.modulation();
        assert_eq!(m.len(), 3);
        assert_eq!(0x34, m[0]);
        assert_eq!(0x12, m[1]);
        assert_eq!(0x78, m[2]);
    }

    #[test]
    fn is_outputting() {
        let mut fpga = FPGAEmulator::new(249);
        assert!(!fpga.is_outputting());

        fpga.normal_op_bram[0] = 0xFFFF;
        assert!(!fpga.is_outputting());

        fpga.modulator_bram[0] = 0xFFFF;
        fpga.controller_bram[ADDR_MOD_CYCLE] = 2 - 1;
        assert!(fpga.is_outputting());
    }

    #[test]
    fn ec_time_now() {
        let t = FPGAEmulator::ec_time_now();
        assert!(t > 0);
    }

    #[test]
    fn ec_time_with_utc_ymdhms() {
        let t = FPGAEmulator::ec_time_with_utc_ymdhms(2000, 1, 1, 0, 0, 0);
        assert!(t.is_ok());
        assert_eq!(t.unwrap(), 0);

        let t = FPGAEmulator::ec_time_with_utc_ymdhms(2000, 1, 1, 0, 0, 1);
        assert!(t.is_ok());
        assert_eq!(t.unwrap(), 1000000000);

        let t = FPGAEmulator::ec_time_with_utc_ymdhms(2001, 1, 1, 0, 0, 0);
        assert!(t.is_ok());
        assert_eq!(t.unwrap(), 31622400000000000);

        let t = FPGAEmulator::ec_time_with_utc_ymdhms(2000, 13, 1, 0, 0, 0);
        assert!(t.is_err());

        let t = FPGAEmulator::ec_time_with_utc_ymdhms(1999, 1, 1, 0, 0, 0);
        assert!(t.is_err());
    }

    #[test]
    fn systime() {
        let t = FPGAEmulator::systime(1000_000_000);
        assert_eq!(t, 20480000);

        let t = FPGAEmulator::systime(2000_000_000);
        assert_eq!(t, 40960000);
    }

    #[test]
    fn stm_idx_from_systime() {
        let stm_cycle = 10;
        let freq_div = 512;

        let mut fpga = FPGAEmulator::new(249);
        {
            let addr =
                ((BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14) | (BRAM_ADDR_STM_CYCLE & 0x3FFF);
            fpga.write(addr, (stm_cycle - 1) as u16);
            assert_eq!(fpga.stm_cycle(), stm_cycle);
        }
        {
            let addr = ((BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14)
                | (BRAM_ADDR_STM_FREQ_DIV_0 & 0x3FFF);
            fpga.write(addr, freq_div as u16);
            assert_eq!(fpga.stm_frequency_division(), freq_div);
        }

        assert_eq!(fpga.stm_idx_from_systime(FPGAEmulator::systime(0)), 0);
        assert_eq!(fpga.stm_idx_from_systime(FPGAEmulator::systime(24_999)), 0);
        assert_eq!(fpga.stm_idx_from_systime(FPGAEmulator::systime(25_000)), 1);
        assert_eq!(
            fpga.stm_idx_from_systime(FPGAEmulator::systime(25_000 * 9)),
            9
        );
        assert_eq!(
            fpga.stm_idx_from_systime(FPGAEmulator::systime(25_000 * 10)),
            0
        );
    }

    #[test]
    fn mod_idx_from_systime() {
        let mod_cycle = 10;
        let freq_div = 512;

        let mut fpga = FPGAEmulator::new(249);
        {
            let addr =
                ((BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14) | (BRAM_ADDR_MOD_CYCLE & 0x3FFF);
            fpga.write(addr, (mod_cycle - 1) as u16);
            assert_eq!(fpga.modulation_cycle(), mod_cycle);
        }
        {
            let addr = ((BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14)
                | (BRAM_ADDR_MOD_FREQ_DIV_0 & 0x3FFF);
            fpga.write(addr, freq_div as u16);
            assert_eq!(fpga.modulation_frequency_division(), freq_div);
        }

        assert_eq!(fpga.mod_idx_from_systime(FPGAEmulator::systime(0)), 0);
        assert_eq!(fpga.mod_idx_from_systime(FPGAEmulator::systime(24_999)), 0);
        assert_eq!(fpga.mod_idx_from_systime(FPGAEmulator::systime(25_000)), 1);
        assert_eq!(
            fpga.mod_idx_from_systime(FPGAEmulator::systime(25_000 * 9)),
            9
        );
        assert_eq!(
            fpga.mod_idx_from_systime(FPGAEmulator::systime(25_000 * 10)),
            0
        );
    }
}
