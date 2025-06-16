use autd3_core::{
    datagram::Segment,
    gain::{Drive, EmitIntensity, Phase},
};

use crate::FPGAEmulator;

impl FPGAEmulator {
    pub(crate) fn gain_stm_drives_inplace(&self, segment: Segment, idx: usize, dst: &mut [Drive]) {
        self.mem.stm_bram[&segment]
            .mem()
            .iter()
            .skip(256 * idx)
            .zip(self.phase_correction().iter())
            .take(self.mem.num_transducers)
            .enumerate()
            .for_each(|(i, (&d, &p))| {
                dst[i] = Drive {
                    phase: Phase((d & 0xFF) as u8) + p,
                    intensity: EmitIntensity(((d >> 8) & 0xFF) as u8),
                }
            })
    }
}
