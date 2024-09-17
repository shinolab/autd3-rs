use autd3_driver::firmware::fpga::Phase;

use super::FPGAEmulator;

impl FPGAEmulator {
    fn _phase_corr(&self, idx: usize) -> Phase {
        let p = &self.mem.phase_corr_bram()[idx >> 1];
        let p = if idx % 2 == 0 { p & 0xFF } else { p >> 8 };
        Phase::new(p as _)
    }

    pub fn phase_correction(&self) -> Vec<Phase> {
        let mut dst = vec![Phase::ZERO; self.mem.num_transducers];
        self.phase_correction_inplace(&mut dst);
        dst
    }

    pub fn phase_correction_inplace(&self, dst: &mut [Phase]) {
        (0..self.mem.num_transducers).for_each(|i| dst[i] = self._phase_corr(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_correction() {
        let fpga = FPGAEmulator::new(249);
        fpga.mem.phase_corr_bram_mut()[0] = 0x1234;
        fpga.mem.phase_corr_bram_mut()[124] = 0x5678;
        assert_eq!(
            [
                vec![Phase::new(0x34), Phase::new(0x12)],
                vec![Phase::ZERO; 246],
                vec![Phase::new(0x78)]
            ]
            .concat(),
            fpga.phase_correction()
        );
    }
}
