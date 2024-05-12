use autd3_driver::{
    derive::{Drive, EmitIntensity, LoopBehavior, Phase, Segment, TransitionMode},
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::fpga::GPIOIn,
};
use num_integer::Roots;

use crate::FPGAEmulator;

use super::super::params::*;

pub(crate) struct Memory {
    num_transducers: usize,
    pub(crate) controller_bram: Vec<u16>,
    pub(crate) modulator_bram_0: Vec<u16>,
    pub(crate) modulator_bram_1: Vec<u16>,
    pub(crate) stm_bram_0: Vec<u16>,
    pub(crate) stm_bram_1: Vec<u16>,
    pub(crate) duty_table_bram: Vec<u16>,
    pub(crate) phase_filter_bram: Vec<u16>,
    pub(crate) drp_bram: Vec<u16>,
    pub(crate) tr_pos: Vec<u64>,
}

impl Memory {
    pub fn new(num_transducers: usize) -> Self {
        Self {
            num_transducers,
            controller_bram: vec![0x0000; 256],
            modulator_bram_0: vec![0x0000; 32768 / std::mem::size_of::<u16>()],
            modulator_bram_1: vec![0x0000; 32768 / std::mem::size_of::<u16>()],
            duty_table_bram: vec![0x0000; 65536 / std::mem::size_of::<u16>()],
            phase_filter_bram: vec![0x0000; 256 / std::mem::size_of::<u16>()],
            stm_bram_0: vec![0x0000; 1024 * 256],
            stm_bram_1: vec![0x0000; 1024 * 256],
            drp_bram: vec![0x0000; 32 * std::mem::size_of::<u64>()],
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

    pub fn init(&mut self) {
        self.controller_bram[ADDR_VERSION_NUM_MAJOR] =
            (ENABLED_FEATURES_BITS as u16) << 8 | VERSION_NUM_MAJOR as u16;
        self.controller_bram[ADDR_VERSION_NUM_MINOR] = VERSION_NUM_MINOR as u16;

        let pwe_init_data = include_bytes!("asin.dat");
        unsafe {
            std::ptr::copy_nonoverlapping(
                pwe_init_data.as_ptr(),
                self.duty_table_bram.as_mut_ptr() as _,
                pwe_init_data.len(),
            );
        }
        self.controller_bram[ADDR_PULSE_WIDTH_ENCODER_FULL_WIDTH_START] = 0xFF * 0xFF;
    }

    pub fn read(&self, addr: u16) -> u16 {
        let select = ((addr >> 14) & 0x0003) as u8;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => self.controller_bram[addr],
            _ => unreachable!(),
        }
    }

    pub fn read_bram_as<T>(bram: &[u16], addr: usize) -> T {
        unsafe { (bram.as_ptr().add(addr) as *const T).read() }
    }

    pub fn write(&mut self, addr: u16, data: u16) {
        let select = ((addr >> 14) & 0x0003) as u8;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => match addr >> 8 {
                BRAM_CNT_SEL_MAIN => self.controller_bram[addr] = data,
                BRAM_CNT_SEL_FILTER => self.phase_filter_bram[addr & 0xFF] = data,
                BRAM_CNT_SEL_CLOCK => self.drp_bram[addr & 0xFF] = data,
                _ => unreachable!(),
            },
            BRAM_SELECT_MOD => match self.controller_bram[ADDR_MOD_MEM_WR_SEGMENT] {
                0 => self.modulator_bram_0[addr] = data,
                1 => self.modulator_bram_1[addr] = data,
                _ => unreachable!(),
            },
            BRAM_SELECT_DUTY_TABLE => {
                self.duty_table_bram[((self.controller_bram[ADDR_PULSE_WIDTH_ENCODER_TABLE_WR_PAGE]
                    as usize)
                    << 14)
                    | addr] = data;
            }
            BRAM_SELECT_STM => match self.controller_bram[ADDR_STM_MEM_WR_SEGMENT] {
                0 => {
                    self.stm_bram_0
                        [(self.controller_bram[ADDR_STM_MEM_WR_PAGE] as usize) << 14 | addr] = data
                }
                1 => {
                    self.stm_bram_1
                        [(self.controller_bram[ADDR_STM_MEM_WR_PAGE] as usize) << 14 | addr] = data
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    pub fn update(&mut self) {
        let mut fpga_state = self.controller_bram[ADDR_FPGA_STATE];
        match self.current_mod_segment() {
            Segment::S0 => fpga_state &= !(1 << 1),
            Segment::S1 => fpga_state |= 1 << 1,
        }
        match self.current_stm_segment() {
            Segment::S0 => fpga_state &= !(1 << 2),
            Segment::S1 => fpga_state |= 1 << 2,
        }
        if self.stm_cycle(self.current_stm_segment()) == 1 {
            fpga_state |= 1 << 3;
        } else {
            fpga_state &= !(1 << 3);
        }
        self.controller_bram[ADDR_FPGA_STATE] = fpga_state;
    }

    pub fn assert_thermal_sensor(&mut self) {
        self.controller_bram[ADDR_FPGA_STATE] |= 1 << 0;
    }

    pub fn deassert_thermal_sensor(&mut self) {
        self.controller_bram[ADDR_FPGA_STATE] &= !(1 << 0);
    }

    pub fn is_force_fan(&self) -> bool {
        (self.controller_bram[ADDR_CTL_FLAG] & (1 << CTL_FLAG_FORCE_FAN_BIT)) != 0
    }

    pub fn gpio_in(&self) -> [bool; 4] {
        [
            (self.controller_bram[ADDR_CTL_FLAG] & (1 << CTL_FLAG_BIT_GPIO_IN_0)) != 0,
            (self.controller_bram[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_1))) != 0,
            (self.controller_bram[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_2))) != 0,
            (self.controller_bram[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_3))) != 0,
        ]
    }

    pub fn is_stm_gain_mode(&self, segment: Segment) -> bool {
        match segment {
            Segment::S0 => self.controller_bram[ADDR_STM_MODE0] == STM_MODE_GAIN,
            Segment::S1 => self.controller_bram[ADDR_STM_MODE1] == STM_MODE_GAIN,
        }
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
        self.controller_bram[ADDR_SILENCER_MODE] == SILNCER_MODE_FIXED_COMPLETION_STEPS
    }

    pub fn stm_freq_division(&self, segment: Segment) -> u32 {
        Self::read_bram_as::<u32>(
            &self.controller_bram,
            match segment {
                Segment::S0 => ADDR_STM_FREQ_DIV0_0,
                Segment::S1 => ADDR_STM_FREQ_DIV1_0,
            },
        )
    }

    pub fn stm_cycle(&self, segment: Segment) -> usize {
        self.controller_bram[match segment {
            Segment::S0 => ADDR_STM_CYCLE0,
            Segment::S1 => ADDR_STM_CYCLE1,
        }] as usize
            + 1
    }

    pub fn sound_speed(&self, segment: Segment) -> u32 {
        Self::read_bram_as::<u32>(
            &self.controller_bram,
            match segment {
                Segment::S0 => ADDR_STM_SOUND_SPEED0_0,
                Segment::S1 => ADDR_STM_SOUND_SPEED1_0,
            },
        )
    }

    pub fn stm_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match Self::read_bram_as::<u32>(
            &self.controller_bram,
            match segment {
                Segment::S0 => ADDR_STM_REP0_0,
                Segment::S1 => ADDR_STM_REP1_0,
            },
        ) {
            0xFFFFFFFF => LoopBehavior::infinite(),
            v => LoopBehavior::finite(v + 1).unwrap(),
        }
    }

    pub fn stm_transition_mode(&self) -> TransitionMode {
        match self.controller_bram[ADDR_STM_TRANSITION_MODE] as u8 {
            TRANSITION_MODE_SYNC_IDX => TransitionMode::SyncIdx,
            TRANSITION_MODE_SYS_TIME => TransitionMode::SysTime(
                DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
                    + std::time::Duration::from_nanos(Self::read_bram_as::<u64>(
                        &self.controller_bram,
                        ADDR_STM_TRANSITION_VALUE_0,
                    )),
            ),
            TRANSITION_MODE_GPIO => TransitionMode::GPIO(
                match Self::read_bram_as::<u64>(&self.controller_bram, ADDR_STM_TRANSITION_VALUE_0)
                {
                    0 => GPIOIn::I0,
                    1 => GPIOIn::I1,
                    2 => GPIOIn::I2,
                    3 => GPIOIn::I3,
                    _ => unreachable!(),
                },
            ),
            TRANSITION_MODE_EXT => TransitionMode::Ext,
            TRANSITION_MODE_IMMIDIATE => TransitionMode::Immidiate,
            _ => unreachable!(),
        }
    }

    pub fn modulation_freq_division(&self, segment: Segment) -> u32 {
        Self::read_bram_as::<u32>(
            &self.controller_bram,
            match segment {
                Segment::S0 => ADDR_MOD_FREQ_DIV0_0,
                Segment::S1 => ADDR_MOD_FREQ_DIV1_0,
            },
        )
    }

    pub fn modulation_cycle(&self, segment: Segment) -> usize {
        self.controller_bram[match segment {
            Segment::S0 => ADDR_MOD_CYCLE0,
            Segment::S1 => ADDR_MOD_CYCLE1,
        }] as usize
            + 1
    }

    pub fn modulation_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match Self::read_bram_as::<u32>(
            &self.controller_bram,
            match segment {
                Segment::S0 => ADDR_MOD_REP0_0,
                Segment::S1 => ADDR_MOD_REP1_0,
            },
        ) {
            0xFFFFFFFF => LoopBehavior::infinite(),
            v => LoopBehavior::finite(v + 1).unwrap(),
        }
    }

    pub fn modulation_at(&self, segment: Segment, idx: usize) -> u8 {
        let m = match segment {
            Segment::S0 => &self.modulator_bram_0[idx >> 1],
            Segment::S1 => &self.modulator_bram_1[idx >> 1],
        };
        let m = if idx % 2 == 0 { m & 0xFF } else { m >> 8 };
        m as u8
    }

    pub fn modulation(&self, segment: Segment) -> Vec<u8> {
        (0..self.modulation_cycle(segment))
            .map(|i| self.modulation_at(segment, i))
            .collect()
    }

    pub fn mod_transition_mode(&self) -> TransitionMode {
        match self.controller_bram[ADDR_MOD_TRANSITION_MODE] as u8 {
            TRANSITION_MODE_SYNC_IDX => TransitionMode::SyncIdx,
            TRANSITION_MODE_SYS_TIME => TransitionMode::SysTime(
                DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
                    + std::time::Duration::from_nanos(Self::read_bram_as::<u64>(
                        &self.controller_bram,
                        ADDR_MOD_TRANSITION_VALUE_0,
                    )),
            ),
            TRANSITION_MODE_GPIO => TransitionMode::GPIO(
                match Self::read_bram_as::<u64>(&self.controller_bram, ADDR_MOD_TRANSITION_VALUE_0)
                {
                    0 => GPIOIn::I0,
                    1 => GPIOIn::I1,
                    2 => GPIOIn::I2,
                    3 => GPIOIn::I3,
                    _ => unreachable!(),
                },
            ),
            TRANSITION_MODE_EXT => TransitionMode::Ext,
            TRANSITION_MODE_IMMIDIATE => TransitionMode::Immidiate,
            _ => unreachable!(),
        }
    }

    pub fn current_mod_segment(&self) -> Segment {
        match self.controller_bram[ADDR_MOD_REQ_RD_SEGMENT] {
            0 => Segment::S0,
            1 => Segment::S1,
            _ => unreachable!(),
        }
    }

    pub fn current_stm_segment(&self) -> Segment {
        match self.controller_bram[ADDR_STM_REQ_RD_SEGMENT] {
            0 => Segment::S0,
            1 => Segment::S1,
            _ => unreachable!(),
        }
    }

    pub fn pulse_width_encoder_table_at(&self, idx: usize) -> u8 {
        let v = self.duty_table_bram[idx >> 1];
        let v = if idx % 2 == 0 { v & 0xFF } else { v >> 8 };
        v as u8
    }

    pub fn pulse_width_encoder_full_width_start(&self) -> u16 {
        self.controller_bram[ADDR_PULSE_WIDTH_ENCODER_FULL_WIDTH_START]
    }

    pub fn pulse_width_encoder_table(&self) -> Vec<u8> {
        self.duty_table_bram
            .iter()
            .flat_map(|&d| vec![(d & 0xFF) as u8, (d >> 8) as u8])
            .collect()
    }

    fn phase_at(&self, idx: usize) -> Phase {
        Phase::new(if idx % 2 == 0 {
            self.phase_filter_bram[idx >> 1] & 0xFF
        } else {
            self.phase_filter_bram[idx >> 1] >> 8
        } as u8)
    }

    pub fn phase_filter(&self) -> Vec<Phase> {
        (0..self.num_transducers)
            .map(|i| self.phase_at(i))
            .collect()
    }

    pub fn debug_types(&self) -> [u8; 4] {
        [
            self.controller_bram[ADDR_DEBUG_TYPE0] as _,
            self.controller_bram[ADDR_DEBUG_TYPE1] as _,
            self.controller_bram[ADDR_DEBUG_TYPE2] as _,
            self.controller_bram[ADDR_DEBUG_TYPE3] as _,
        ]
    }

    pub fn debug_values(&self) -> [u16; 4] {
        [
            self.controller_bram[ADDR_DEBUG_VALUE0],
            self.controller_bram[ADDR_DEBUG_VALUE1],
            self.controller_bram[ADDR_DEBUG_VALUE2],
            self.controller_bram[ADDR_DEBUG_VALUE3],
        ]
    }

    pub fn drp_rom(&self) -> Vec<u64> {
        unsafe { std::slice::from_raw_parts(self.drp_bram.as_ptr() as *const u64, 32).to_vec() }
    }

    pub fn drives(&self, segment: Segment, idx: usize) -> Vec<Drive> {
        if self.is_stm_gain_mode(segment) {
            self.gain_stm_drives(segment, idx)
        } else {
            self.focus_stm_drives(segment, idx)
        }
    }

    fn gain_stm_drives(&self, segment: Segment, idx: usize) -> Vec<Drive> {
        match segment {
            Segment::S0 => &self.stm_bram_0,
            Segment::S1 => &self.stm_bram_1,
        }
        .iter()
        .skip(256 * idx)
        .take(self.num_transducers)
        .zip(self.phase_filter())
        .map(|(&d, p)| {
            Drive::new(
                Phase::new((d & 0xFF) as u8) + p,
                EmitIntensity::new(((d >> 8) & 0xFF) as u8),
            )
        })
        .collect()
    }

    fn focus_stm_drives(&self, segment: Segment, idx: usize) -> Vec<Drive> {
        let bram = match segment {
            Segment::S0 => &self.stm_bram_0,
            Segment::S1 => &self.stm_bram_1,
        };
        let sound_speed = self.sound_speed(segment);

        let intensity = bram[4 * idx + 3] >> 6 & 0x00FF;

        let mut x = (bram[4 * idx + 1] as u32) << 16 & 0x30000;
        x |= bram[4 * idx] as u32;
        let x = if (x & 0x20000) != 0 {
            (x | 0xFFFC0000) as i32
        } else {
            x as i32
        };
        let mut y = (bram[4 * idx + 2] as u32) << 14 & 0x3C000;
        y |= bram[4 * idx + 1] as u32 >> 2;
        let y = if (y & 0x20000) != 0 {
            (y | 0xFFFC0000) as i32
        } else {
            y as i32
        };
        let mut z = (bram[4 * idx + 3] as u32) << 12 & 0x3F000;
        z |= bram[4 * idx + 2] as u32 >> 4;
        let z = if (z & 0x20000) != 0 {
            (z | 0xFFFC0000) as i32
        } else {
            z as i32
        };
        self.tr_pos
            .iter()
            .zip(self.phase_filter())
            .map(|(&tr, p)| {
                let tr_z = ((tr >> 32) & 0xFFFF) as i16 as i32;
                let tr_x = ((tr >> 16) & 0xFFFF) as i16 as i32;
                let tr_y = (tr & 0xFFFF) as i16 as i32;
                let d2 =
                    (x - tr_x) * (x - tr_x) + (y - tr_y) * (y - tr_y) + (z - tr_z) * (z - tr_z);
                let dist = d2.sqrt() as u64;
                let q = (dist << 18) / sound_speed as u64;
                Drive::new(
                    Phase::new((q & 0xFF) as u8) + p,
                    EmitIntensity::new(intensity as u8),
                )
            })
            .collect()
    }

    pub fn local_tr_pos(&self) -> &[u64] {
        &self.tr_pos
    }
}

impl FPGAEmulator {
    pub(crate) fn read(&self, addr: u16) -> u16 {
        self.mem.read(addr)
    }

    pub(crate) fn write(&mut self, addr: u16, data: u16) {
        self.mem.write(addr, data)
    }

    pub fn update(&mut self) {
        self.mem.update()
    }

    pub fn assert_thermal_sensor(&mut self) {
        self.mem.assert_thermal_sensor()
    }

    pub fn deassert_thermal_sensor(&mut self) {
        self.mem.deassert_thermal_sensor()
    }

    pub fn is_force_fan(&self) -> bool {
        self.mem.is_force_fan()
    }

    pub fn gpio_in(&self) -> [bool; 4] {
        self.mem.gpio_in()
    }

    pub fn is_stm_gain_mode(&self, segment: Segment) -> bool {
        self.mem.is_stm_gain_mode(segment)
    }

    pub fn silencer_update_rate_intensity(&self) -> u16 {
        self.mem.silencer_update_rate_intensity()
    }

    pub fn silencer_update_rate_phase(&self) -> u16 {
        self.mem.silencer_update_rate_phase()
    }

    pub fn silencer_completion_steps_intensity(&self) -> u16 {
        self.mem.silencer_completion_steps_intensity()
    }

    pub fn silencer_completion_steps_phase(&self) -> u16 {
        self.mem.silencer_completion_steps_phase()
    }

    pub fn silencer_fixed_completion_steps_mode(&self) -> bool {
        self.mem.silencer_fixed_completion_steps_mode()
    }

    pub fn stm_freq_division(&self, segment: Segment) -> u32 {
        self.mem.stm_freq_division(segment)
    }

    pub fn stm_cycle(&self, segment: Segment) -> usize {
        self.mem.stm_cycle(segment)
    }

    pub fn sound_speed(&self, segment: Segment) -> u32 {
        self.mem.sound_speed(segment)
    }

    pub fn stm_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        self.mem.stm_loop_behavior(segment)
    }

    pub fn stm_transition_mode(&self) -> TransitionMode {
        self.mem.stm_transition_mode()
    }

    pub fn modulation_freq_division(&self, segment: Segment) -> u32 {
        self.mem.modulation_freq_division(segment)
    }

    pub fn modulation_cycle(&self, segment: Segment) -> usize {
        self.mem.modulation_cycle(segment)
    }

    pub fn modulation_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        self.mem.modulation_loop_behavior(segment)
    }

    pub fn modulation_at(&self, segment: Segment, idx: usize) -> u8 {
        self.mem.modulation_at(segment, idx)
    }

    pub fn modulation(&self, segment: Segment) -> Vec<u8> {
        self.mem.modulation(segment)
    }

    pub fn mod_transition_mode(&self) -> TransitionMode {
        self.mem.mod_transition_mode()
    }

    pub fn current_mod_segment(&self) -> Segment {
        self.mem.current_mod_segment()
    }

    pub fn current_stm_segment(&self) -> Segment {
        self.mem.current_stm_segment()
    }

    pub fn pulse_width_encoder_table_at(&self, idx: usize) -> u8 {
        self.mem.pulse_width_encoder_table_at(idx)
    }

    pub fn pulse_width_encoder_full_width_start(&self) -> u16 {
        self.mem.pulse_width_encoder_full_width_start()
    }

    pub fn pulse_width_encoder_table(&self) -> Vec<u8> {
        self.mem.pulse_width_encoder_table()
    }

    pub fn phase_filter(&self) -> Vec<Phase> {
        self.mem.phase_filter()
    }

    pub fn debug_types(&self) -> [u8; 4] {
        self.mem.debug_types()
    }

    pub fn debug_values(&self) -> [u16; 4] {
        self.mem.debug_values()
    }

    pub fn drp_rom(&self) -> Vec<u64> {
        self.mem.drp_rom()
    }

    pub fn drives(&self, segment: Segment, idx: usize) -> Vec<Drive> {
        self.mem.drives(segment, idx)
    }

    pub fn local_tr_pos(&self) -> &[u64] {
        self.mem.local_tr_pos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn read_panic() {
        let fpga = FPGAEmulator::new(249);
        let addr = (BRAM_SELECT_MOD as u16) << 14;
        fpga.read(addr as _);
    }

    #[test]
    fn modulation() {
        let mut fpga = FPGAEmulator::new(249);
        fpga.mem.modulator_bram_0[0] = 0x1234;
        fpga.mem.modulator_bram_0[1] = 0x5678;
        fpga.mem.controller_bram[ADDR_MOD_CYCLE0] = 3 - 1;
        assert_eq!(3, fpga.modulation_cycle(Segment::S0));
        assert_eq!(0x34, fpga.modulation_at(Segment::S0, 0));
        assert_eq!(0x12, fpga.modulation_at(Segment::S0, 1));
        assert_eq!(0x78, fpga.modulation_at(Segment::S0, 2));
        let m = fpga.modulation(Segment::S0);
        assert_eq!(3, m.len());
        assert_eq!(0x34, m[0]);
        assert_eq!(0x12, m[1]);
        assert_eq!(0x78, m[2]);
    }

    #[test]
    fn is_outputting() {
        let mut fpga = FPGAEmulator::new(249);
        fpga.mem.controller_bram[ADDR_STM_MODE0] = STM_MODE_GAIN;

        assert!(!fpga.is_outputting());

        fpga.mem.stm_bram_0[0] = 0xFFFF;
        assert!(!fpga.is_outputting());

        fpga.mem.modulator_bram_0[0] = 0xFFFF;
        fpga.mem.controller_bram[ADDR_MOD_CYCLE0] = 2 - 1;
        assert!(fpga.is_outputting());
    }
}
