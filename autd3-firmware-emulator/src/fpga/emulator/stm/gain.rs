use autd3_driver::{
    derive::Segment,
    firmware::fpga::{Drive, EmitIntensity, Phase},
};

use crate::FPGAEmulator;

impl FPGAEmulator {
    pub(crate) fn gain_stm_drives_inplace(&self, segment: Segment, idx: usize, dst: &mut [Drive]) {
        self.mem.stm_bram()[&segment]
            .iter()
            .skip(256 * idx)
            .zip(self.phase_correction().iter())
            .take(self.mem.num_transducers)
            .enumerate()
            .for_each(|(i, (&d, &p))| {
                dst[i] = Drive::new(
                    Phase::new((d & 0xFF) as u8) + p,
                    EmitIntensity::new(((d >> 8) & 0xFF) as u8),
                )
            })
    }
}
