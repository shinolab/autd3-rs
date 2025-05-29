use std::num::NonZeroU16;

use autd3_driver::{
    ethercat::DcSysTime,
    firmware::fpga::{Drive, GPIOIn, LoopBehavior, Segment, TransitionMode},
};

use super::{super::params::*, FPGAEmulator};

mod foci;
mod gain;

impl FPGAEmulator {
    #[must_use]
    pub fn is_stm_gain_mode(&self, segment: Segment) -> bool {
        self.mem
            .controller_bram
            .read(ADDR_STM_MODE0 + segment as usize)
            == STM_MODE_GAIN
    }

    #[must_use]
    pub fn stm_freq_divide(&self, segment: Segment) -> u16 {
        self.mem
            .controller_bram
            .read(ADDR_STM_FREQ_DIV0 + segment as usize)
    }

    #[must_use]
    pub fn stm_cycle(&self, segment: Segment) -> usize {
        self.mem
            .controller_bram
            .read(ADDR_STM_CYCLE0 + segment as usize) as usize
            + 1
    }

    #[must_use]
    pub fn stm_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match self
            .mem
            .controller_bram
            .read(ADDR_STM_REP0 + segment as usize)
        {
            0xFFFF => LoopBehavior::Infinite,
            v => LoopBehavior::Finite(NonZeroU16::new(v + 1).unwrap()),
        }
    }

    #[must_use]
    pub fn stm_transition_mode(&self) -> TransitionMode {
        match self.mem.controller_bram.read(ADDR_STM_TRANSITION_MODE) as u8 {
            TRANSITION_MODE_SYNC_IDX => TransitionMode::SyncIdx,
            TRANSITION_MODE_SYS_TIME => TransitionMode::SysTime(
                DcSysTime::ZERO
                    + std::time::Duration::from_nanos(
                        self.mem
                            .controller_bram
                            .read_bram_as::<u64>(ADDR_STM_TRANSITION_VALUE_0),
                    ),
            ),
            TRANSITION_MODE_GPIO => TransitionMode::GPIO(
                match self
                    .mem
                    .controller_bram
                    .read_bram_as::<u64>(ADDR_STM_TRANSITION_VALUE_0)
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

    #[must_use]
    pub fn req_stm_segment(&self) -> Segment {
        match self.mem.controller_bram.read(ADDR_STM_REQ_RD_SEGMENT) {
            0 => Segment::S0,
            1 => Segment::S1,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn drives(&self) -> Vec<Drive> {
        self.drives_at(self.current_stm_segment(), self.current_stm_idx())
    }

    #[must_use]
    pub fn drives_at(&self, segment: Segment, idx: usize) -> Vec<Drive> {
        let mut dst = vec![Drive::NULL; self.mem.num_transducers];
        self.drives_at_inplace(segment, idx, &mut dst);
        dst
    }

    pub fn drives_at_inplace(&self, segment: Segment, idx: usize, dst: &mut [Drive]) {
        if self.is_stm_gain_mode(segment) {
            self.gain_stm_drives_inplace(segment, idx, dst)
        } else {
            self.foci_stm_drives_inplace(segment, idx, dst)
        }
    }
}
