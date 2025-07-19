use autd3_core::{
    datagram::{Segment, transition_mode::TransitionModeParams},
    gain::{Drive, Phase},
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
    pub fn stm_loop_count(&self, segment: Segment) -> u16 {
        self.mem
            .controller_bram
            .read(ADDR_STM_REP0 + segment as usize)
    }

    #[must_use]
    pub fn stm_transition_mode(&self) -> TransitionModeParams {
        TransitionModeParams {
            mode: self.mem.controller_bram.read(ADDR_STM_TRANSITION_MODE) as u8,
            value: self
                .mem
                .controller_bram
                .read_bram_as::<u64>(ADDR_STM_TRANSITION_VALUE_0),
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
        let mut phase_corr_buf = vec![Phase::ZERO; self.mem.num_transducers];
        let mut output_mask_buf = vec![false; self.mem.num_transducers];
        let mut dst = vec![Drive::NULL; self.mem.num_transducers];
        self.drives_at_inplace(
            segment,
            idx,
            &mut phase_corr_buf,
            &mut output_mask_buf,
            &mut dst,
        );
        dst
    }

    pub fn drives_at_inplace(
        &self,
        segment: Segment,
        idx: usize,
        phase_corr_buf: &mut [Phase],
        output_mask_buf: &mut [bool],
        dst: &mut [Drive],
    ) {
        if self.is_stm_gain_mode(segment) {
            self.gain_stm_drives_inplace(segment, idx, phase_corr_buf, output_mask_buf, dst)
        } else {
            self.foci_stm_drives_inplace(segment, idx, phase_corr_buf, output_mask_buf, dst)
        }
    }
}
