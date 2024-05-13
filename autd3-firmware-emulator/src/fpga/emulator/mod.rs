mod memory;
mod swapchain;

use autd3_driver::{
    defined::Freq,
    defined::FREQ_40K,
    derive::{EmitIntensity, Segment},
    ethercat::DcSysTime,
    firmware::fpga::ULTRASOUND_PERIOD,
};

use memory::Memory;

use super::params::{CTL_FLAG_MOD_SET_BIT, CTL_FLAG_STM_SET_BIT};

const CTL_FLAG_MOD_SET: u16 = 1 << CTL_FLAG_MOD_SET_BIT;
const CTL_FLAG_STM_SET: u16 = 1 << CTL_FLAG_STM_SET_BIT;

pub struct FPGAEmulator {
    mem: Memory,
    mod_swapchain: swapchain::Swapchain<CTL_FLAG_MOD_SET>,
    stm_swapchain: swapchain::Swapchain<CTL_FLAG_STM_SET>,
    fpga_clk_freq: Freq<u32>,
}

impl FPGAEmulator {
    pub(crate) fn new(num_transducers: usize) -> Self {
        let mut fpga = Self {
            mem: Memory::new(num_transducers),
            mod_swapchain: swapchain::Swapchain::new(),
            stm_swapchain: swapchain::Swapchain::new(),
            fpga_clk_freq: FREQ_40K * ULTRASOUND_PERIOD,
        };
        fpga.init();
        fpga
    }

    pub(crate) fn init(&mut self) {
        self.mem.init();
        self.mod_swapchain.init();
        self.stm_swapchain.init();
    }

    pub(crate) fn write(&mut self, addr: u16, data: u16) {
        self.mem.write(addr, data);
    }

    pub(crate) fn set_and_wait_update(&mut self, sys_time: DcSysTime) {
        let addr = ((crate::fpga::params::BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14)
            | (crate::fpga::params::ADDR_CTL_FLAG as u16 & 0x3FFF);
        if (self.read(addr) & CTL_FLAG_MOD_SET) == CTL_FLAG_MOD_SET {
            self.mod_swapchain.set(
                sys_time,
                self.mem
                    .modulation_loop_behavior(self.mem.req_mod_segment()),
                self.mem
                    .modulation_freq_division(self.mem.req_mod_segment()),
                self.mem.modulation_cycle(self.mem.req_mod_segment()),
                self.mem.req_mod_segment(),
                self.mem.mod_transition_mode(),
            );
        }
        if (self.read(addr) & CTL_FLAG_STM_SET) == CTL_FLAG_STM_SET {
            self.stm_swapchain.set(
                sys_time,
                self.mem.stm_loop_behavior(self.mem.req_stm_segment()),
                self.mem.stm_freq_division(self.mem.req_stm_segment()),
                self.mem.stm_cycle(self.mem.req_stm_segment()),
                self.mem.req_stm_segment(),
                self.mem.stm_transition_mode(),
            );
        }
    }

    pub fn update(&mut self) {
        self.update_with_sys_time(DcSysTime::now());
    }

    pub fn update_with_sys_time(&mut self, sys_time: DcSysTime) {
        self.mod_swapchain.update(self.mem.gpio_in(), sys_time);
        self.stm_swapchain.update(self.mem.gpio_in(), sys_time);

        let mut fpga_state = self.mem.fpga_state();
        match self.current_mod_segment() {
            Segment::S0 => fpga_state &= !(1 << 1),
            Segment::S1 => fpga_state |= 1 << 1,
        }
        match self.current_stm_segment() {
            Segment::S0 => fpga_state &= !(1 << 2),
            Segment::S1 => fpga_state |= 1 << 2,
        }
        if self.stm_cycle(self.current_stm_segment()) == 1 {
            fpga_state |= 1 << 3;
        } else {
            fpga_state &= !(1 << 3);
        }
        self.mem.update(fpga_state);
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

    pub(crate) fn set_fpga_clk_freq(&mut self, freq: Freq<u32>) {
        self.fpga_clk_freq = freq;
        self.mod_swapchain.fpga_clk_freq = freq;
        self.stm_swapchain.fpga_clk_freq = freq;
    }

    pub fn fpga_clk_freq(&self) -> Freq<u32> {
        self.fpga_clk_freq
    }

    fn fpga_sys_time(&self, dc_sys_time: DcSysTime) -> u64 {
        ((dc_sys_time.sys_time() as u128 * self.fpga_clk_freq().hz() as u128) / 1000000000) as _
    }

    #[deprecated(note = "Use `current_stm_idx` instead", since = "23.1.0")]
    pub fn stm_idx_from_systime(&self, segment: Segment, sys_time: DcSysTime) -> usize {
        (self.fpga_sys_time(sys_time) / self.stm_freq_division(segment) as u64) as usize
            % self.stm_cycle(segment)
    }

    #[deprecated(note = "Use `current_mod_idx` instead", since = "23.1.0")]
    pub fn mod_idx_from_systime(&self, segment: Segment, sys_time: DcSysTime) -> usize {
        (self.fpga_sys_time(sys_time) / self.modulation_freq_division(segment) as u64) as usize
            % self.modulation_cycle(segment)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

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
    fn sys_time(#[case] expect: u64, #[case] value: u64) {
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
}
