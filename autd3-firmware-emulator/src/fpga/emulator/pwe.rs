use autd3_core::{datagram::PulseWidth, gain::Intensity};

use super::FPGAEmulator;

impl FPGAEmulator {
    #[must_use]
    pub fn pulse_width_encoder_table_at(&self, idx: usize) -> PulseWidth {
        PulseWidth::new(self.mem.duty_table_bram.read(idx) as _)
    }

    #[must_use]
    pub fn pulse_width_encoder_table(&self) -> Vec<PulseWidth> {
        let mut dst = vec![PulseWidth::new(0); 256];
        self.pulse_width_encoder_table_inplace(&mut dst);
        dst
    }

    pub fn pulse_width_encoder_table_inplace(&self, dst: &mut [PulseWidth]) {
        dst.iter_mut().enumerate().for_each(|(i, d)| {
            *d = self.pulse_width_encoder_table_at(i);
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
    fn test_to_pulse_width() {
        let fpga = FPGAEmulator::new(249);
        itertools::iproduct!(0x00..=0xFF, 0x00..=0xFF).for_each(|(a, b)| {
            assert_eq!(
                to_pulse_width_actual(a, b),
                fpga.to_pulse_width(Intensity(a), b)
            );
        });
    }
}
