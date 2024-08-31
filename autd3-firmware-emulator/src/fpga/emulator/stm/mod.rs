use autd3_driver::{
    derive::{LoopBehavior, Segment, TransitionMode},
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::fpga::{Drive, GPIOIn},
};

use super::{super::params::*, memory::Memory, FPGAEmulator};

mod foci;
mod gain;

impl FPGAEmulator {
    pub fn is_stm_gain_mode(&self, segment: Segment) -> bool {
        match segment {
            Segment::S0 => self.mem.controller_bram()[ADDR_STM_MODE0] == STM_MODE_GAIN,
            Segment::S1 => self.mem.controller_bram()[ADDR_STM_MODE1] == STM_MODE_GAIN,
            _ => unimplemented!(),
        }
    }

    pub fn stm_freq_division(&self, segment: Segment) -> u16 {
        Memory::read_bram_as::<u16>(
            &self.mem.controller_bram(),
            match segment {
                Segment::S0 => ADDR_STM_FREQ_DIV0,
                Segment::S1 => ADDR_STM_FREQ_DIV1,
                _ => unimplemented!(),
            },
        )
    }

    pub fn stm_cycle(&self, segment: Segment) -> usize {
        self.mem.controller_bram()[match segment {
            Segment::S0 => ADDR_STM_CYCLE0,
            Segment::S1 => ADDR_STM_CYCLE1,
            _ => unimplemented!(),
        }] as usize
            + 1
    }

    pub fn stm_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match Memory::read_bram_as::<u16>(
            &self.mem.controller_bram(),
            match segment {
                Segment::S0 => ADDR_STM_REP0,
                Segment::S1 => ADDR_STM_REP1,
                _ => unimplemented!(),
            },
        ) {
            0xFFFF => LoopBehavior::infinite(),
            v => LoopBehavior::finite(v + 1).unwrap(),
        }
    }

    pub fn stm_transition_mode(&self) -> TransitionMode {
        match self.mem.controller_bram()[ADDR_STM_TRANSITION_MODE] as u8 {
            TRANSITION_MODE_SYNC_IDX => TransitionMode::SyncIdx,
            TRANSITION_MODE_SYS_TIME => TransitionMode::SysTime(
                DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
                    + std::time::Duration::from_nanos(Memory::read_bram_as::<u64>(
                        &self.mem.controller_bram(),
                        ADDR_STM_TRANSITION_VALUE_0,
                    )),
            ),
            TRANSITION_MODE_GPIO => TransitionMode::GPIO(
                match Memory::read_bram_as::<u64>(
                    &self.mem.controller_bram(),
                    ADDR_STM_TRANSITION_VALUE_0,
                ) {
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

    pub fn req_stm_segment(&self) -> Segment {
        match self.mem.controller_bram()[ADDR_STM_REQ_RD_SEGMENT] {
            0 => Segment::S0,
            1 => Segment::S1,
            _ => unreachable!(),
        }
    }

    pub fn drives(&self) -> Vec<Drive> {
        self.drives_at(self.current_stm_segment(), self.current_stm_idx())
    }

    pub fn drives_at(&self, segment: Segment, idx: usize) -> Vec<Drive> {
        if self.is_stm_gain_mode(segment) {
            self.gain_stm_drives(segment, idx)
        } else {
            self.foci_stm_drives(segment, idx)
        }
    }
}
