use autd3_driver::firmware::fpga::{Drive, EmitIntensity, Phase, Segment};

use super::super::{super::params::*, FPGAEmulator};

#[bitfield_struct::bitfield(u64)]
struct STMFocus {
    #[bits(18)]
    pub x: i32,
    #[bits(18)]
    pub y: i32,
    #[bits(18)]
    pub z: i32,
    #[bits(8)]
    pub intensity: u8,
    #[bits(2)]
    __: u8,
}

impl FPGAEmulator {
    pub fn sound_speed(&self, segment: Segment) -> u16 {
        self.mem.controller_bram.borrow()[match segment {
            Segment::S0 => ADDR_STM_SOUND_SPEED0,
            Segment::S1 => ADDR_STM_SOUND_SPEED1,
        }]
    }

    pub fn num_foci(&self, segment: Segment) -> u8 {
        self.mem.controller_bram.borrow()[match segment {
            Segment::S0 => ADDR_STM_NUM_FOCI0,
            Segment::S1 => ADDR_STM_NUM_FOCI1,
        }] as u8
    }

    pub(crate) fn foci_stm_drives_inplace(&self, segment: Segment, idx: usize, dst: &mut [Drive]) {
        let bram = &self.mem.stm_bram.borrow()[&segment];
        let sound_speed = self.sound_speed(segment);

        self.mem
            .tr_pos
            .iter()
            .zip(self.phase_correction().iter())
            .take(self.mem.num_transducers)
            .enumerate()
            .for_each(|(i, (&tr, &p))| {
                let tr_z = ((tr >> 32) & 0xFFFF) as i16 as i32;
                let tr_x = ((tr >> 16) & 0xFFFF) as i16 as i32;
                let tr_y = (tr & 0xFFFF) as i16 as i32;
                let mut intensity = 0x00;
                let (sin, cos) = (0..self.num_foci(segment) as usize).fold((0, 0), |acc, i| {
                    let f = unsafe {
                        (bram[32 * idx + 4 * i..].as_ptr() as *const STMFocus).read_unaligned()
                    };
                    let x = f.x();
                    let y = f.y();
                    let z = f.z();
                    let intensity_or_offset = f.intensity();
                    let offset = if i == 0 {
                        intensity = intensity_or_offset;
                        0x00
                    } else {
                        intensity_or_offset
                    };

                    let d2 =
                        (x - tr_x) * (x - tr_x) + (y - tr_y) * (y - tr_y) + (z - tr_z) * (z - tr_z);
                    let dist = d2.isqrt() as u32;
                    let q = ((dist << 14) / sound_speed as u32) as usize;
                    let q = q + offset as usize;
                    (
                        acc.0 + self.mem.sin_table[q % 256] as u16,
                        acc.1 + self.mem.sin_table[(q + 64) % 256] as u16,
                    )
                });
                let sin = ((sin / self.num_foci(segment) as u16) >> 1) as usize;
                let cos = ((cos / self.num_foci(segment) as u16) >> 1) as usize;
                let phase = self.mem.atan_table[(sin << 7) | cos];
                dst[i] = Drive {
                    phase: Phase(phase) + p,
                    intensity: EmitIntensity(intensity),
                };
            });
    }

    pub fn local_tr_pos(&self) -> &[u64] {
        &self.mem.tr_pos
    }
}
