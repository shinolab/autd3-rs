use autd3_driver::firmware::fpga::EmitIntensity;

use super::FPGAEmulator;

impl FPGAEmulator {
    pub fn pulse_width_encoder_table_at(&self, idx: usize) -> u8 {
        let v = self.mem.duty_table_bram()[idx >> 1];
        let v = if idx % 2 == 0 { v & 0xFF } else { v >> 8 };
        v as u8
    }

    pub fn pulse_width_encoder_table(&self) -> Vec<u8> {
        self.mem
            .duty_table_bram()
            .iter()
            .flat_map(|&d| vec![(d & 0xFF) as u8, (d >> 8) as u8])
            .collect()
    }

    pub fn to_pulse_width(&self, a: EmitIntensity, b: u8) -> u8 {
        let key = (a.value() as usize * b as usize) / 255;
        self.pulse_width_encoder_table_at(key)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    static ASIN_TABLE: &[u8; 256] = include_bytes!("asin.dat");

    fn to_pulse_width_actual(a: u8, b: u8) -> u8 {
        let idx = (a as usize * b as usize) / 255;
        ASIN_TABLE[idx]
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_to_pulse_width() {
        let fpga = FPGAEmulator::new(249);
        itertools::iproduct!(0x00..=0xFF, 0x00..=0xFF).for_each(|(a, b)| {
            assert_eq!(
                to_pulse_width_actual(a, b),
                fpga.to_pulse_width(a.into(), b)
            );
        });
    }
}