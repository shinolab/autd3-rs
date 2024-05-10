use autd3_driver::{
    ethercat::DcSysTime,
    firmware::cpu::{Header, RxMessage, TxDatagram},
};

use crate::fpga::emulator::FPGAEmulator;

use super::params::*;

pub struct CPUEmulator {
    pub(crate) idx: usize,
    pub(crate) ack: u8,
    pub(crate) last_msg_id: u8,
    pub(crate) rx_data: u8,
    pub(crate) read_fpga_state: bool,
    pub(crate) read_fpga_state_store: bool,
    pub(crate) mod_cycle: u32,
    pub(crate) stm_cycle: [u32; 2],
    pub(crate) stm_mode: [u16; 2],
    pub(crate) stm_rep: [u32; 2],
    pub(crate) stm_freq_div: [u32; 2],
    pub(crate) stm_segment: u8,
    pub(crate) stm_transition_mode: u8,
    pub(crate) stm_transition_value: u64,
    pub(crate) mod_freq_div: [u32; 2],
    pub(crate) mod_segment: u8,
    pub(crate) mod_rep: [u32; 2],
    pub(crate) mod_transition_mode: u8,
    pub(crate) mod_transition_value: u64,
    pub(crate) gain_stm_mode: u8,
    pub(crate) fpga: FPGAEmulator,
    pub(crate) synchronized: bool,
    pub(crate) num_transducers: usize,
    pub(crate) fpga_flags_internal: u16,
    pub(crate) silencer_strict_mode: bool,
    pub(crate) min_freq_div_intensity: u32,
    pub(crate) min_freq_div_phase: u32,
    pub(crate) is_rx_data_used: bool,
    pub(crate) pwe_write: u32,
    pub(crate) dc_sys_time: DcSysTime,
    pub(crate) clk_write: u32,
}

impl CPUEmulator {
    pub fn new(id: usize, num_transducers: usize) -> Self {
        let mut s = Self {
            idx: id,
            ack: 0x00,
            last_msg_id: 0x00,
            rx_data: 0x00,
            read_fpga_state: false,
            read_fpga_state_store: false,
            mod_cycle: 0,
            stm_cycle: [1, 1],
            stm_mode: [STM_MODE_GAIN, STM_MODE_GAIN],
            gain_stm_mode: 0,
            stm_transition_mode: TRANSITION_MODE_SYNC_IDX,
            stm_transition_value: 0,
            mod_transition_mode: TRANSITION_MODE_SYNC_IDX,
            mod_transition_value: 0,
            fpga: FPGAEmulator::new(num_transducers),
            synchronized: false,
            num_transducers,
            fpga_flags_internal: 0x0000,
            mod_freq_div: [5120, 5120],
            mod_segment: 0,
            stm_freq_div: [0xFFFFFFFF, 0xFFFFFFFF],
            stm_segment: 0,
            silencer_strict_mode: true,
            min_freq_div_intensity: 5120,
            min_freq_div_phase: 20480,
            is_rx_data_used: false,
            pwe_write: 0,
            dc_sys_time: DcSysTime::now(),
            clk_write: 0,
            stm_rep: [0xFFFFFFFF, 0xFFFFFFFF],
            mod_rep: [0xFFFFFFFF, 0xFFFFFFFF],
        };
        s.init();
        s
    }

    pub const fn idx(&self) -> usize {
        self.idx
    }

    pub const fn num_transducers(&self) -> usize {
        self.num_transducers
    }

    pub const fn synchronized(&self) -> bool {
        self.synchronized
    }

    pub const fn reads_fpga_state(&self) -> bool {
        self.read_fpga_state
    }

    pub const fn rx(&self) -> RxMessage {
        RxMessage::new(self.ack, self.rx_data)
    }

    pub const fn fpga(&self) -> &FPGAEmulator {
        &self.fpga
    }

    pub fn fpga_mut(&mut self) -> &mut FPGAEmulator {
        &mut self.fpga
    }

    pub fn send(&mut self, tx: &TxDatagram) {
        self.ecat_recv(tx.data(self.idx));
    }

    pub fn init(&mut self) {
        self.fpga.init();
        unsafe {
            self.clear(&[]);
        }
    }

    pub fn update(&mut self) {
        if self.should_update() {
            self.fpga.update();
            self.read_fpga_state();
        }
        self.dc_sys_time = DcSysTime::now();
    }

    pub fn set_dc_sys_time(&mut self, dc_sys_time: DcSysTime) {
        self.dc_sys_time = dc_sys_time;
    }

    pub fn dc_sys_time(&self) -> DcSysTime {
        self.dc_sys_time
    }

    pub const fn should_update(&self) -> bool {
        self.read_fpga_state
    }

    pub const fn silencer_strict_mode(&self) -> bool {
        self.silencer_strict_mode
    }
}

impl CPUEmulator {
    pub(crate) const fn cast<T>(data: &[u8]) -> &T {
        unsafe { &*(data.as_ptr() as *const T) }
    }

    const fn get_addr(select: u8, addr: u16) -> u16 {
        ((select as u16 & 0x0003) << 14) | (addr & 0x3FFF)
    }

    pub(crate) fn bram_read(&self, select: u8, addr: u16) -> u16 {
        let addr = Self::get_addr(select, addr);
        self.fpga.read(addr)
    }

    pub(crate) fn bram_write(&mut self, select: u8, addr: u16, data: u16) {
        let addr = Self::get_addr(select, addr);
        self.fpga.write(addr, data)
    }

    pub(crate) fn bram_cpy(&mut self, select: u8, addr_base: u16, data: *const u16, size: usize) {
        let mut addr = Self::get_addr(select, addr_base);
        let mut src = data;
        (0..size).for_each(|_| unsafe {
            self.fpga.write(addr, src.read());
            addr += 1;
            src = src.add(1);
        })
    }

    pub(crate) fn bram_set(&mut self, select: u8, addr_base: u16, value: u16, size: usize) {
        let mut addr = Self::get_addr(select, addr_base);
        (0..size).for_each(|_| {
            self.fpga.write(addr, value);
            addr += 1;
        })
    }

    fn read_fpga_state(&mut self) {
        if self.is_rx_data_used {
            return;
        }
        if self.read_fpga_state {
            self.rx_data = FPGA_STATE_READS_FPGA_STATE_ENABLED
                | self.bram_read(BRAM_SELECT_CONTROLLER, ADDR_FPGA_STATE) as u8;
        } else {
            self.rx_data &= !FPGA_STATE_READS_FPGA_STATE_ENABLED;
        }
    }

    fn handle_payload(&mut self, data: &[u8]) -> u8 {
        unsafe {
            match data[0] {
                TAG_CLEAR => self.clear(data),
                TAG_SYNC => self.synchronize(data),
                TAG_FIRM_INFO => self.firm_info(data),
                TAG_CONFIG_FPGA_CLK => self.configure_clk(data),
                TAG_MODULATION => self.write_mod(data),
                TAG_MODULATION_CHANGE_SEGMENT => self.change_mod_segment(data),
                TAG_SILENCER => self.config_silencer(data),
                TAG_GAIN => self.write_gain(data),
                TAG_GAIN_CHANGE_SEGMENT => self.change_gain_segment(data),
                TAG_GAIN_STM_CHANGE_SEGMENT => self.change_gain_stm_segment(data),
                TAG_FOCUS_STM => self.write_focus_stm(data),
                TAG_FOCUS_STM_CHANGE_SEGMENT => self.change_focus_stm_segment(data),
                TAG_GAIN_STM => self.write_gain_stm(data),
                TAG_FORCE_FAN => self.configure_force_fan(data),
                TAG_READS_FPGA_STATE => self.configure_reads_fpga_state(data),
                TAG_CONFIG_PULSE_WIDTH_ENCODER => self.config_pwe(data),
                TAG_PHASE_FILTER => self.write_phase_filter(data),
                TAG_DEBUG => self.config_debug(data),
                TAG_EMULATE_GPIO_IN => self.emulate_gpio_in(data),
                _ => ERR_NOT_SUPPORTED_TAG,
            }
        }
    }

    fn ecat_recv(&mut self, data: &[u8]) {
        let header = unsafe { &*(data.as_ptr() as *const Header) };

        if self.ack == header.msg_id {
            return;
        }
        self.last_msg_id = header.msg_id;

        self.read_fpga_state();

        if (header.msg_id & 0x80) != 0 {
            self.ack = ERR_INVALID_MSG_ID;
            return;
        }

        self.ack = self.handle_payload(&data[std::mem::size_of::<Header>()..]);
        if (self.ack & ERR_BIT) != 0 {
            return;
        }

        if header.slot_2_offset != 0 {
            self.ack = self.handle_payload(
                &data[std::mem::size_of::<Header>() + header.slot_2_offset as usize..],
            );
            if (self.ack & ERR_BIT) != 0 {
                return;
            }
        }

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_CTL_FLAG,
            self.fpga_flags_internal,
        );

        self.ack = header.msg_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    #[test]
    fn cpu_idx() {
        let mut rng = rand::thread_rng();
        let idx = rng.gen();
        let cpu = CPUEmulator::new(idx, 249);
        assert_eq!(idx, cpu.idx());
    }

    #[test]
    fn num_transducers() {
        let cpu = CPUEmulator::new(0, 249);
        assert_eq!(249, cpu.num_transducers());
    }

    #[test]
    fn dc_sys_time() {
        let mut cpu = CPUEmulator::new(0, 249);

        let sys_time = DcSysTime::now() + std::time::Duration::from_nanos(1111);
        cpu.set_dc_sys_time(sys_time);
        assert_eq!(sys_time, cpu.dc_sys_time());
    }
}
