use autd3_core::firmware::{Drive, Intensity, Phase, Segment};

use crate::FPGAEmulator;

impl FPGAEmulator {
    pub(crate) fn gain_stm_drives_inplace(
        &self,
        segment: Segment,
        idx: usize,
        phase_corr_buf: &mut [Phase],
        output_mask_buf: &mut [bool],
        dst: &mut [Drive],
    ) {
        self.phase_correction_inplace(phase_corr_buf);
        self.output_mask_inplace(segment, output_mask_buf);

        self.mem.stm_bram[&segment]
            .mem()
            .iter()
            .skip(256 * idx)
            .zip(phase_corr_buf.iter())
            .zip(output_mask_buf.iter())
            .take(self.mem.num_transducers)
            .enumerate()
            .for_each(|(i, ((&d, &p), &mask))| {
                dst[i] = Drive {
                    phase: Phase((d & 0xFF) as u8) + p,
                    intensity: Intensity(if mask { ((d >> 8) & 0xFF) as u8 } else { 0x00 }),
                }
            })
    }
}
