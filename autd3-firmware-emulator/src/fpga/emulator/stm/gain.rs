use autd3_driver::{
    derive::Segment,
    firmware::fpga::{Drive, EmitIntensity, Phase},
};

use crate::FPGAEmulator;

impl FPGAEmulator {
    pub(crate) fn gain_stm_drives_inplace(&self, segment: Segment, idx: usize, dst: &mut [Drive]) {
        match segment {
            Segment::S0 => self.mem.stm_bram_0(),
            Segment::S1 => self.mem.stm_bram_1(),
            _ => unimplemented!(),
        }
        .iter()
        .skip(256 * idx)
        .take(self.mem.num_transducers)
        .enumerate()
        .for_each(|(i, &d)| {
            dst[i] = Drive::new(
                Phase::new((d & 0xFF) as u8),
                EmitIntensity::new(((d >> 8) & 0xFF) as u8),
            )
        })
    }
}
