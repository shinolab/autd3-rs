use autd3_core::firmware::Phase;

use super::FPGAEmulator;

impl FPGAEmulator {
    #[must_use]
    fn _phase_corr(&self, idx: usize) -> Phase {
        let p = &self.mem.phase_corr_bram.read(idx >> 1);
        let p = if idx.is_multiple_of(2) {
            p & 0xFF
        } else {
            p >> 8
        };
        Phase(p as _)
    }

    #[must_use]
    pub fn phase_correction(&self) -> Vec<Phase> {
        let len = self.mem.num_transducers;
        let mut buffer = Vec::with_capacity(len);
        unsafe {
            self.phase_correction_inplace(buffer.as_mut_ptr());
            buffer.set_len(len);
        }
        buffer
    }

    pub unsafe fn phase_correction_inplace(&self, dst: *mut Phase) {
        (0..self.mem.num_transducers).for_each(|i| {
            unsafe { dst.add(i).write(self._phase_corr(i)) };
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_correction() {
        let fpga = FPGAEmulator::new(249);
        fpga.mem.phase_corr_bram.write(0, 0x1234);
        fpga.mem.phase_corr_bram.write(124, 0x5678);
        assert_eq!(
            [
                vec![Phase(0x34), Phase(0x12)],
                vec![Phase::ZERO; 246],
                vec![Phase(0x78)]
            ]
            .concat(),
            fpga.phase_correction()
        );
    }
}
