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
    controller_bram: Vec<u16>,
    modulator_bram_0: Vec<u16>,
    modulator_bram_1: Vec<u16>,
    stm_bram_0: Vec<u16>,
    stm_bram_1: Vec<u16>,
    duty_table_bram: Vec<u16>,
    drp_bram: Vec<u16>,
    tr_pos: Vec<u64>,
    sin_table: Vec<u8>,
    atan_table: Vec<u8>,
}

impl Memory {
    pub fn new(num_transducers: usize) -> Self {
        Self {
            num_transducers,
            controller_bram: vec![0x0000; 256],
            modulator_bram_0: vec![0x0000; 32768 / std::mem::size_of::<u16>()],
            modulator_bram_1: vec![0x0000; 32768 / std::mem::size_of::<u16>()],
            duty_table_bram: vec![0x0000; 32768 / std::mem::size_of::<u16>()],
            stm_bram_0: vec![0x0000; 1024 * 256],
            stm_bram_1: vec![0x0000; 1024 * 256],
            drp_bram: vec![0x0000; 32 * std::mem::size_of::<u64>()],
            tr_pos: vec![
                0x00000000, 0x01960000, 0x032d0000, 0x04c30000, 0x065a0000, 0x07f00000, 0x09860000,
                0x0b1d0000, 0x0cb30000, 0x0e4a0000, 0x0fe00000, 0x11760000, 0x130d0000, 0x14a30000,
                0x163a0000, 0x17d00000, 0x19660000, 0x1afd0000, 0x00000196, 0x04c30196, 0x065a0196,
                0x07f00196, 0x09860196, 0x0b1d0196, 0x0cb30196, 0x0e4a0196, 0x0fe00196, 0x11760196,
                0x130d0196, 0x14a30196, 0x163a0196, 0x17d00196, 0x1afd0196, 0x0000032d, 0x0196032d,
                0x032d032d, 0x04c3032d, 0x065a032d, 0x07f0032d, 0x0986032d, 0x0b1d032d, 0x0cb3032d,
                0x0e4a032d, 0x0fe0032d, 0x1176032d, 0x130d032d, 0x14a3032d, 0x163a032d, 0x17d0032d,
                0x1966032d, 0x1afd032d, 0x000004c3, 0x019604c3, 0x032d04c3, 0x04c304c3, 0x065a04c3,
                0x07f004c3, 0x098604c3, 0x0b1d04c3, 0x0cb304c3, 0x0e4a04c3, 0x0fe004c3, 0x117604c3,
                0x130d04c3, 0x14a304c3, 0x163a04c3, 0x17d004c3, 0x196604c3, 0x1afd04c3, 0x0000065a,
                0x0196065a, 0x032d065a, 0x04c3065a, 0x065a065a, 0x07f0065a, 0x0986065a, 0x0b1d065a,
                0x0cb3065a, 0x0e4a065a, 0x0fe0065a, 0x1176065a, 0x130d065a, 0x14a3065a, 0x163a065a,
                0x17d0065a, 0x1966065a, 0x1afd065a, 0x000007f0, 0x019607f0, 0x032d07f0, 0x04c307f0,
                0x065a07f0, 0x07f007f0, 0x098607f0, 0x0b1d07f0, 0x0cb307f0, 0x0e4a07f0, 0x0fe007f0,
                0x117607f0, 0x130d07f0, 0x14a307f0, 0x163a07f0, 0x17d007f0, 0x196607f0, 0x1afd07f0,
                0x00000986, 0x01960986, 0x032d0986, 0x04c30986, 0x065a0986, 0x07f00986, 0x09860986,
                0x0b1d0986, 0x0cb30986, 0x0e4a0986, 0x0fe00986, 0x11760986, 0x130d0986, 0x14a30986,
                0x163a0986, 0x17d00986, 0x19660986, 0x1afd0986, 0x00000b1d, 0x01960b1d, 0x032d0b1d,
                0x04c30b1d, 0x065a0b1d, 0x07f00b1d, 0x09860b1d, 0x0b1d0b1d, 0x0cb30b1d, 0x0e4a0b1d,
                0x0fe00b1d, 0x11760b1d, 0x130d0b1d, 0x14a30b1d, 0x163a0b1d, 0x17d00b1d, 0x19660b1d,
                0x1afd0b1d, 0x00000cb3, 0x01960cb3, 0x032d0cb3, 0x04c30cb3, 0x065a0cb3, 0x07f00cb3,
                0x09860cb3, 0x0b1d0cb3, 0x0cb30cb3, 0x0e4a0cb3, 0x0fe00cb3, 0x11760cb3, 0x130d0cb3,
                0x14a30cb3, 0x163a0cb3, 0x17d00cb3, 0x19660cb3, 0x1afd0cb3, 0x00000e4a, 0x01960e4a,
                0x032d0e4a, 0x04c30e4a, 0x065a0e4a, 0x07f00e4a, 0x09860e4a, 0x0b1d0e4a, 0x0cb30e4a,
                0x0e4a0e4a, 0x0fe00e4a, 0x11760e4a, 0x130d0e4a, 0x14a30e4a, 0x163a0e4a, 0x17d00e4a,
                0x19660e4a, 0x1afd0e4a, 0x00000fe0, 0x01960fe0, 0x032d0fe0, 0x04c30fe0, 0x065a0fe0,
                0x07f00fe0, 0x09860fe0, 0x0b1d0fe0, 0x0cb30fe0, 0x0e4a0fe0, 0x0fe00fe0, 0x11760fe0,
                0x130d0fe0, 0x14a30fe0, 0x163a0fe0, 0x17d00fe0, 0x19660fe0, 0x1afd0fe0, 0x00001176,
                0x01961176, 0x032d1176, 0x04c31176, 0x065a1176, 0x07f01176, 0x09861176, 0x0b1d1176,
                0x0cb31176, 0x0e4a1176, 0x0fe01176, 0x11761176, 0x130d1176, 0x14a31176, 0x163a1176,
                0x17d01176, 0x19661176, 0x1afd1176, 0x0000130d, 0x0196130d, 0x032d130d, 0x04c3130d,
                0x065a130d, 0x07f0130d, 0x0986130d, 0x0b1d130d, 0x0cb3130d, 0x0e4a130d, 0x0fe0130d,
                0x1176130d, 0x130d130d, 0x14a3130d, 0x163a130d, 0x17d0130d, 0x1966130d, 0x1afd130d,
                0x000014a3, 0x019614a3, 0x032d14a3, 0x04c314a3, 0x065a14a3, 0x07f014a3, 0x098614a3,
                0x0b1d14a3, 0x0cb314a3, 0x0e4a14a3, 0x0fe014a3, 0x117614a3, 0x130d14a3, 0x14a314a3,
                0x163a14a3, 0x17d014a3, 0x196614a3, 0x1afd14a3, 0x00000000, 0x00000000, 0x00000000,
                0x00000000, 0x00000000, 0x00000000, 0x00000000,
            ],
            sin_table: include_bytes!("sin.dat").to_vec(),
            atan_table: include_bytes!("atan.dat").to_vec(),
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

    pub fn update(&mut self, fpga_state: u16) {
        self.controller_bram[ADDR_FPGA_STATE] = fpga_state;
    }

    pub fn fpga_state(&self) -> u16 {
        self.controller_bram[ADDR_FPGA_STATE]
    }

    pub fn assert_thermal_sensor(&mut self) {
        self.controller_bram[ADDR_FPGA_STATE] |= 1 << 0;
    }

    pub fn deassert_thermal_sensor(&mut self) {
        self.controller_bram[ADDR_FPGA_STATE] &= !(1 << 0);
    }

    pub fn is_thermo_asserted(&self) -> bool {
        (self.controller_bram[ADDR_FPGA_STATE] & (1 << 0)) != 0
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
            _ => unimplemented!(),
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
                _ => unimplemented!(),
            },
        )
    }

    pub fn stm_cycle(&self, segment: Segment) -> usize {
        self.controller_bram[match segment {
            Segment::S0 => ADDR_STM_CYCLE0,
            Segment::S1 => ADDR_STM_CYCLE1,
            _ => unimplemented!(),
        }] as usize
            + 1
    }

    pub fn sound_speed(&self, segment: Segment) -> u16 {
        self.controller_bram[match segment {
            Segment::S0 => ADDR_STM_SOUND_SPEED0,
            Segment::S1 => ADDR_STM_SOUND_SPEED1,
            _ => unimplemented!(),
        }]
    }

    pub fn num_foci(&self, segment: Segment) -> u8 {
        self.controller_bram[match segment {
            Segment::S0 => ADDR_STM_NUM_FOCI0,
            Segment::S1 => ADDR_STM_NUM_FOCI1,
            _ => unimplemented!(),
        }] as u8
    }

    pub fn stm_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match Self::read_bram_as::<u32>(
            &self.controller_bram,
            match segment {
                Segment::S0 => ADDR_STM_REP0_0,
                Segment::S1 => ADDR_STM_REP1_0,
                _ => unimplemented!(),
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
            TRANSITION_MODE_IMMEDIATE => TransitionMode::Immediate,
            _ => unreachable!(),
        }
    }

    pub fn modulation_freq_division(&self, segment: Segment) -> u32 {
        Self::read_bram_as::<u32>(
            &self.controller_bram,
            match segment {
                Segment::S0 => ADDR_MOD_FREQ_DIV0_0,
                Segment::S1 => ADDR_MOD_FREQ_DIV1_0,
                _ => unimplemented!(),
            },
        )
    }

    pub fn modulation_cycle(&self, segment: Segment) -> usize {
        self.controller_bram[match segment {
            Segment::S0 => ADDR_MOD_CYCLE0,
            Segment::S1 => ADDR_MOD_CYCLE1,
            _ => unimplemented!(),
        }] as usize
            + 1
    }

    pub fn modulation_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match Self::read_bram_as::<u32>(
            &self.controller_bram,
            match segment {
                Segment::S0 => ADDR_MOD_REP0_0,
                Segment::S1 => ADDR_MOD_REP1_0,
                _ => unimplemented!(),
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
            _ => unimplemented!(),
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
            TRANSITION_MODE_IMMEDIATE => TransitionMode::Immediate,
            _ => unreachable!(),
        }
    }

    pub fn req_mod_segment(&self) -> Segment {
        match self.controller_bram[ADDR_MOD_REQ_RD_SEGMENT] {
            0 => Segment::S0,
            1 => Segment::S1,
            _ => unreachable!(),
        }
    }

    pub fn req_stm_segment(&self) -> Segment {
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
            self.foci_stm_drives(segment, idx)
        }
    }

    fn gain_stm_drives(&self, segment: Segment, idx: usize) -> Vec<Drive> {
        match segment {
            Segment::S0 => &self.stm_bram_0,
            Segment::S1 => &self.stm_bram_1,
            _ => unimplemented!(),
        }
        .iter()
        .skip(256 * idx)
        .take(self.num_transducers)
        .map(|&d| {
            Drive::new(
                Phase::new((d & 0xFF) as u8),
                EmitIntensity::new(((d >> 8) & 0xFF) as u8),
            )
        })
        .collect()
    }

    fn foci_stm_drives(&self, segment: Segment, idx: usize) -> Vec<Drive> {
        let bram = match segment {
            Segment::S0 => &self.stm_bram_0,
            Segment::S1 => &self.stm_bram_1,
            _ => unimplemented!(),
        };
        let sound_speed = self.sound_speed(segment);

        let intensity = (bram[32 * idx + 3] >> 6 & 0x00FF) as u8;

        self.tr_pos
            .iter()
            .map(|&tr| {
                let tr_z = ((tr >> 32) & 0xFFFF) as i16 as i32;
                let tr_x = ((tr >> 16) & 0xFFFF) as i16 as i32;
                let tr_y = (tr & 0xFFFF) as i16 as i32;
                let (sin, cos) = (0..self.num_foci(segment) as usize).fold((0, 0), |acc, i| {
                    let mut x = (bram[32 * idx + 4 * i + 1] as u32) << 16 & 0x30000;
                    x |= bram[32 * idx + 4 * i] as u32;
                    let x = if (x & 0x20000) != 0 {
                        (x | 0xFFFC0000) as i32
                    } else {
                        x as i32
                    };
                    let mut y = (bram[32 * idx + 4 * i + 2] as u32) << 14 & 0x3C000;
                    y |= bram[32 * idx + 4 * i + 1] as u32 >> 2;
                    let y = if (y & 0x20000) != 0 {
                        (y | 0xFFFC0000) as i32
                    } else {
                        y as i32
                    };
                    let mut z = (bram[32 * idx + 4 * i + 3] as u32) << 12 & 0x3F000;
                    z |= bram[32 * idx + 4 * i + 2] as u32 >> 4;
                    let z = if (z & 0x20000) != 0 {
                        (z | 0xFFFC0000) as i32
                    } else {
                        z as i32
                    };
                    let offset = bram[32 * idx + 4 * i + 3] >> 6 & 0x00FF;

                    let d2 =
                        (x - tr_x) * (x - tr_x) + (y - tr_y) * (y - tr_y) + (z - tr_z) * (z - tr_z);
                    let dist = d2.sqrt() as u32;
                    let q = ((dist << 14) / sound_speed as u32) as usize;
                    let q = if i == 0 { q } else { q + offset as usize };
                    (
                        acc.0 + self.sin_table[q % 256] as u16,
                        acc.1 + self.sin_table[(q + 64) % 256] as u16,
                    )
                });
                let sin = ((sin / self.num_foci(segment) as u16) >> 1) as usize;
                let cos = ((cos / self.num_foci(segment) as u16) >> 1) as usize;
                let phase = self.atan_table[(sin << 7) | cos];
                Drive::new(Phase::new(phase), EmitIntensity::new(intensity))
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

    pub fn sound_speed(&self, segment: Segment) -> u16 {
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

    pub fn req_mod_segment(&self) -> Segment {
        self.mem.req_mod_segment()
    }

    pub fn req_stm_segment(&self) -> Segment {
        self.mem.req_stm_segment()
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
