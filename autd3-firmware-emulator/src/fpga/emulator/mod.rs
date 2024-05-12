mod memory;

use autd3_driver::{
    defined::Freq,
    defined::FREQ_40K,
    derive::{EmitIntensity, Segment},
    ethercat::DcSysTime,
    firmware::fpga::ULTRASOUND_PERIOD,
};

use memory::Memory;

pub struct FPGAEmulator {
    mem: Memory,
    pub(crate) fpga_clk_freq: Freq<u32>,
}

impl FPGAEmulator {
    pub(crate) fn new(num_transducers: usize) -> Self {
        let mut fpga = Self {
            mem: Memory::new(num_transducers),
            fpga_clk_freq: FREQ_40K * ULTRASOUND_PERIOD,
        };
        fpga.init();
        fpga
    }

    pub(crate) fn init(&mut self) {
        self.mem.init();
    }

    pub fn to_pulse_width(&self, a: EmitIntensity, b: u8) -> u16 {
        let key = a.value() as usize * b as usize;
        let v = self.pulse_width_encoder_table_at(key) as u16;
        if key as u16 >= self.pulse_width_encoder_full_width_start() {
            0x100 | v
        } else {
            v
        }
    }

    pub fn is_outputting(&self) -> bool {
        let cur_mod_segment = self.current_mod_segment();
        if self
            .modulation(cur_mod_segment)
            .iter()
            .all(|&m| m == u8::MIN)
        {
            return false;
        }
        let cur_stm_segment = self.current_stm_segment();
        (0..self.stm_cycle(cur_stm_segment)).any(|i| {
            self.drives(cur_stm_segment, i)
                .iter()
                .any(|&d| d.intensity() != EmitIntensity::MIN)
        })
    }

    pub fn ultrasound_freq(&self) -> Freq<u32> {
        self.fpga_clk_freq / ULTRASOUND_PERIOD
    }

    pub fn fpga_clk_freq(&self) -> Freq<u32> {
        self.fpga_clk_freq
    }

    fn fpga_sys_time(&self, dc_sys_time: DcSysTime) -> u64 {
        ((dc_sys_time.sys_time() as u128 * self.fpga_clk_freq().hz() as u128) / 1000000000) as _
    }

    pub fn stm_idx_from_systime(&self, segment: Segment, systime: DcSysTime) -> usize {
        (self.fpga_sys_time(systime) / self.stm_freq_division(segment) as u64) as usize
            % self.stm_cycle(segment)
    }

    pub fn mod_idx_from_systime(&self, segment: Segment, systime: DcSysTime) -> usize {
        (self.fpga_sys_time(systime) / self.modulation_freq_division(segment) as u64) as usize
            % self.modulation_cycle(segment)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    use crate::fpga::params::*;
    use autd3_driver::ethercat::ECAT_DC_SYS_TIME_BASE;

    static ASIN_TABLE: &[u8; 65536] = include_bytes!("asin.dat");

    fn to_pulse_width_actual(a: u8, b: u8) -> u16 {
        let r = ASIN_TABLE[a as usize * b as usize];
        let full_width = a == 0xFF && b == 0xFF;
        if full_width {
            r as u16 | 0x0100
        } else {
            r as u16
        }
    }

    #[test]
    fn test_to_pulse_width() {
        let fpga = FPGAEmulator::new(249);
        itertools::iproduct!(0x00..=0xFF, 0x00..=0xFF).for_each(|(a, b)| {
            assert_eq!(
                to_pulse_width_actual(a, b),
                fpga.to_pulse_width(a.into(), b)
            );
        });
    }

    #[rstest::rstest]
    #[test]
    #[case(20480000, 1_000_000_000)]
    #[case(40960000, 2_000_000_000)]
    fn systime(#[case] expect: u64, #[case] value: u64) {
        let fpga = FPGAEmulator::new(249);
        assert_eq!(
            expect,
            fpga.fpga_sys_time(
                DcSysTime::from_utc(
                    ECAT_DC_SYS_TIME_BASE + std::time::Duration::from_nanos(value),
                )
                .unwrap(),
            )
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(0, 24_999)]
    #[case(1, 25_000)]
    #[case(9, 25_000 * 9)]
    #[case(0, 25_000 * 10)]
    fn stm_idx_from_systime(#[case] expect: usize, #[case] value: u64) {
        let stm_cycle = 10;
        let freq_div = 512;

        let mut fpga = FPGAEmulator::new(249);
        {
            let addr = ((BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14)
                | (ADDR_STM_CYCLE0 as u16 & 0x3FFF);
            fpga.write(addr, (stm_cycle - 1) as u16);
            assert_eq!(stm_cycle, fpga.stm_cycle(Segment::S0));
        }
        {
            let addr = ((BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14)
                | (ADDR_STM_FREQ_DIV0_0 as u16 & 0x3FFF);
            fpga.write(addr, freq_div as u16);
            assert_eq!(freq_div, fpga.stm_freq_division(Segment::S0));
        }

        assert_eq!(
            expect,
            fpga.stm_idx_from_systime(Segment::S0, DcSysTime::from_utc(
                ECAT_DC_SYS_TIME_BASE + std::time::Duration::from_nanos(value),
            )
            .unwrap())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(0, 24_999)]
    #[case(1, 25_000)]
    #[case(9, 25_000 * 9)]
    #[case(0, 25_000 * 10)]
    fn mod_idx_from_systime(#[case] expect: usize, #[case] value: u64) {
        let mod_cycle = 10;
        let freq_div = 512;

        let mut fpga = FPGAEmulator::new(249);
        {
            let addr = ((BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14)
                | (ADDR_MOD_CYCLE0 as u16 & 0x3FFF);
            fpga.write(addr, (mod_cycle - 1) as u16);
            assert_eq!(mod_cycle, fpga.modulation_cycle(Segment::S0));
        }
        {
            let addr = ((BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14)
                | (ADDR_MOD_FREQ_DIV0_0 as u16 & 0x3FFF);
            fpga.write(addr, freq_div as u16);
            assert_eq!(freq_div, fpga.modulation_freq_division(Segment::S0));
        }

        assert_eq!(
            expect,
            fpga.mod_idx_from_systime(Segment::S0, DcSysTime::from_utc(
                ECAT_DC_SYS_TIME_BASE + std::time::Duration::from_nanos(value),
            )
            .unwrap())
        );
    }
}
