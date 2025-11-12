use autd3_core::firmware::{Drive, Intensity, Phase, Segment};

use super::super::{super::params::*, FPGAEmulator};

pub struct STMFocus(u64);

impl STMFocus {
    fn x(&self) -> i32 {
        let x = (self.0 & 0x3_FFFF) as u32;
        if x & 0x2_0000 != 0 {
            (x | 0xFFFC_0000) as i32
        } else {
            x as i32
        }
    }

    fn y(&self) -> i32 {
        let y = ((self.0 >> 18) & 0x3_FFFF) as u32;
        if y & 0x2_0000 != 0 {
            (y | 0xFFFC_0000) as i32
        } else {
            y as i32
        }
    }

    fn z(&self) -> i32 {
        let z = ((self.0 >> 36) & 0x3_FFFF) as u32;
        if z & 0x2_0000 != 0 {
            (z | 0xFFFC_0000) as i32
        } else {
            z as i32
        }
    }

    fn intensity(&self) -> u8 {
        ((self.0 >> 54) & 0xFF) as u8
    }
}

impl FPGAEmulator {
    #[must_use]
    pub fn sound_speed(&self, segment: Segment) -> u16 {
        self.mem
            .controller_bram
            .read(ADDR_STM_SOUND_SPEED0 + segment as usize)
    }

    #[must_use]
    pub fn num_foci(&self, segment: Segment) -> u8 {
        self.mem
            .controller_bram
            .read(ADDR_STM_NUM_FOCI0 + segment as usize) as u8
    }

    pub(crate) fn foci_stm_drives_inplace(
        &self,
        segment: Segment,
        idx: usize,
        phase_corr_buf: *mut Phase,
        output_mask_buf: *mut bool,
        dst: *mut Drive,
    ) {
        let bram = &self.mem.stm_bram[&segment];
        let sound_speed = self.sound_speed(segment);
        let num_foci = self.num_foci(segment) as usize;

        unsafe {
            self.phase_correction_inplace(phase_corr_buf);
            self.output_mask_inplace(segment, output_mask_buf)
        };

        self.local_tr_pos()
            .iter()
            .take(self.mem.num_transducers)
            .enumerate()
            .for_each(|(i, tr)| {
                let tr_z = ((tr >> 32) & 0xFFFF) as i16 as i32;
                let tr_x = ((tr >> 16) & 0xFFFF) as i16 as i32;
                let tr_y = (tr & 0xFFFF) as i16 as i32;
                let mut intensity = 0x00;
                let (sin, cos) = (0..num_foci).fold((0, 0), |acc, i| {
                    let f = bram.read_bram_as::<STMFocus>(
                        size_of::<STMFocus>() / size_of::<u16>() * (idx * num_foci + i),
                    );
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
                let sin = ((sin / num_foci as u16) >> 1) as usize;
                let cos = ((cos / num_foci as u16) >> 1) as usize;
                let phase = self.mem.atan_table[(sin << 7) | cos];
                unsafe {
                    let p = phase_corr_buf.add(i).read();
                    let mask = output_mask_buf.add(i).read();
                    dst.add(i).write(Drive {
                        phase: Phase(phase) + p,
                        intensity: Intensity(if mask { intensity } else { 0x00 }),
                    })
                };
            });
    }

    #[must_use]
    pub fn local_tr_pos(&self) -> &[u64] {
        self.mem.tr_pos.mem.as_slice()
    }

    #[must_use]
    pub fn local_tr_pos_at(&self, idx: usize) -> u64 {
        self.mem.tr_pos[idx]
    }
}
