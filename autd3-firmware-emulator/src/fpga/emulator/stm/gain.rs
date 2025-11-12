use autd3_core::firmware::{Drive, Intensity, Phase, Segment};

use crate::FPGAEmulator;

impl FPGAEmulator {
    pub(crate) fn gain_stm_drives_inplace(
        &self,
        segment: Segment,
        idx: usize,
        phase_corr_buf: *mut Phase,
        output_mask_buf: *mut bool,
        dst: *mut Drive,
    ) {
        unsafe {
            self.phase_correction_inplace(phase_corr_buf);
            self.output_mask_inplace(segment, output_mask_buf);
        }

        self.mem.stm_bram[&segment]
            .mem()
            .iter()
            .skip(256 * idx)
            .take(self.mem.num_transducers)
            .enumerate()
            .for_each(|(i, d)| unsafe {
                let p = phase_corr_buf.add(i).read();
                let mask = output_mask_buf.add(i).read();
                dst.add(i).write(Drive {
                    phase: Phase((d & 0xFF) as u8) + p,
                    intensity: Intensity(if mask { ((d >> 8) & 0xFF) as u8 } else { 0x00 }),
                });
            })
    }
}
