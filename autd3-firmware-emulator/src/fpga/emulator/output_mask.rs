use autd3_core::firmware::Segment;

use super::FPGAEmulator;

impl FPGAEmulator {
    #[must_use]
    fn _output_mask(&self, idx: usize) -> bool {
        let chunk = idx >> 4;
        let idx = idx & 0x0F;
        let p = &self.mem.output_mask_bram.read(chunk);
        *p & (1 << idx) != 0
    }

    #[must_use]
    pub fn output_mask(&self, segment: Segment) -> Vec<bool> {
        let len = self.mem.num_transducers;
        let mut buffer = Vec::with_capacity(len);
        unsafe {
            self.output_mask_inplace(segment, buffer.as_mut_ptr());
            buffer.set_len(len);
        }
        buffer
    }

    pub unsafe fn output_mask_inplace(&self, segment: Segment, dst: *mut bool) {
        (0..self.mem.num_transducers).for_each(|i| {
            unsafe {
                dst.add(i)
                    .write(self._output_mask(i | (segment as usize) << 8))
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_mask() {
        let fpga = FPGAEmulator::new(249);
        fpga.mem.output_mask_bram.write(0, 0b1111_1111_1111_0000);
        fpga.mem.output_mask_bram.write(31, 0b1111_1110_0001_1111);
        assert_eq!(
            [vec![false; 4], vec![true; 245]].concat(),
            fpga.output_mask(Segment::S0)
        );
        assert_eq!(
            [vec![true; 245], vec![false; 4],].concat(),
            fpga.output_mask(Segment::S1)
        );
    }
}
