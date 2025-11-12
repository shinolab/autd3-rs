use autd3_core::firmware::{Intensity, PulseWidth};

use super::FPGAEmulator;

impl FPGAEmulator {
    #[must_use]
    pub fn pulse_width_encoder_table_at(&self, idx: usize) -> PulseWidth {
        PulseWidth::new(self.mem.duty_table_bram.read(idx) as _)
    }

    #[must_use]
    pub fn pulse_width_encoder_table(&self) -> Vec<PulseWidth> {
        let mut buffer = Vec::with_capacity(256);
        unsafe {
            self.pulse_width_encoder_table_inplace(buffer.as_mut_ptr());
            buffer.set_len(256);
        }
        buffer
    }

    pub unsafe fn pulse_width_encoder_table_inplace(&self, dst: *mut PulseWidth) {
        (0..256).for_each(|i| {
            unsafe { dst.add(i).write(self.pulse_width_encoder_table_at(i)) };
        });
    }

    #[must_use]
    pub fn to_pulse_width(&self, a: Intensity, b: u8) -> PulseWidth {
        let key = (a.0 as usize * b as usize) / 255;
        self.pulse_width_encoder_table_at(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static ASIN_TABLE: &[u8; 512] = include_bytes!("asin.dat");

    fn to_pulse_width_actual(a: u8, b: u8) -> PulseWidth {
        let idx = (a as usize * b as usize) / 255;
        PulseWidth::new(u16::from_le_bytes([ASIN_TABLE[(idx << 1) + 1], ASIN_TABLE[idx << 1]]) as _)
    }

    #[test]
    fn to_pulse_width() {
        let fpga = FPGAEmulator::new(249);
        (0x00..=0xFF).for_each(|a| {
            (0x00..=0xFF).for_each(|b| {
                assert_eq!(
                    to_pulse_width_actual(a, b),
                    fpga.to_pulse_width(Intensity(a), b)
                );
            });
        });
    }
}
