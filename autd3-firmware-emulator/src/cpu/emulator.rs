use autd3_driver::{
    ethercat::{DcSysTime, EC_OUTPUT_FRAME_SIZE},
    firmware::cpu::{Header, RxMessage, TxMessage},
    link::{Ack, MsgId},
};

use getset::{CopyGetters, Getters, MutGetters};

use crate::fpga::emulator::FPGAEmulator;

use super::params::*;

#[derive(CopyGetters, Getters, MutGetters)]
pub struct CPUEmulator {
    #[getset(get_copy = "pub")]
    pub(crate) idx: usize,
    pub(crate) err: u8,
    pub(crate) last_msg_id: MsgId,
    pub(crate) rx_data: u8,
    #[getset(get_copy = "pub")]
    pub(crate) reads_fpga_state: bool,
    pub(crate) reads_fpga_state_store: bool,
    pub(crate) mod_cycle: u32,
    pub(crate) stm_write: u32,
    pub(crate) stm_cycle: [u32; 2],
    pub(crate) stm_mode: [u16; 2],
    pub(crate) stm_rep: [u16; 2],
    pub(crate) stm_freq_div: [u16; 2],
    pub(crate) stm_segment: u8,
    pub(crate) stm_transition_mode: u8,
    pub(crate) stm_transition_value: u64,
    pub(crate) num_foci: u8,
    pub(crate) mod_freq_div: [u16; 2],
    pub(crate) mod_segment: u8,
    pub(crate) mod_rep: [u16; 2],
    pub(crate) mod_transition_mode: u8,
    pub(crate) mod_transition_value: u64,
    pub(crate) gain_stm_mode: u8,
    #[getset(get = "pub", get_mut = "pub")]
    pub(crate) fpga: FPGAEmulator,
    #[getset(get_copy = "pub")]
    pub(crate) synchronized: bool,
    #[getset(get_copy = "pub")]
    pub(crate) num_transducers: usize,
    pub(crate) fpga_flags_internal: u16,
    #[getset(get_copy = "pub")]
    pub(crate) silencer_strict_mode: bool,
    pub(crate) min_freq_div_intensity: u16,
    pub(crate) min_freq_div_phase: u16,
    pub(crate) is_rx_data_used: bool,
    #[getset(get_copy = "pub")]
    pub(crate) dc_sys_time: DcSysTime,
    #[getset(get_copy = "pub")]
    pub(crate) port_a_podr: u8,
    pub(crate) broken: bool,
}

impl CPUEmulator {
    #[must_use]
    pub fn new(id: usize, num_transducers: usize) -> Self {
        let mut s = Self {
            idx: id,
            err: 0x00,
            last_msg_id: MsgId::new(0xFF),
            rx_data: 0x00,
            reads_fpga_state: false,
            reads_fpga_state_store: false,
            mod_cycle: 0,
            stm_cycle: [1, 1],
            stm_mode: [STM_MODE_GAIN, STM_MODE_GAIN],
            gain_stm_mode: 0,
            stm_transition_mode: TRANSITION_MODE_SYNC_IDX,
            stm_transition_value: 0,
            num_foci: 1,
            mod_transition_mode: TRANSITION_MODE_SYNC_IDX,
            mod_transition_value: 0,
            fpga: FPGAEmulator::new(num_transducers),
            synchronized: false,
            num_transducers,
            fpga_flags_internal: 0x0000,
            mod_freq_div: [10, 10],
            mod_segment: 0,
            stm_freq_div: [0xFFFF, 0xFFFF],
            stm_segment: 0,
            stm_write: 0,
            silencer_strict_mode: true,
            min_freq_div_intensity: 10,
            min_freq_div_phase: 40,
            is_rx_data_used: false,
            dc_sys_time: DcSysTime::now(),
            stm_rep: [0xFFFF, 0xFFFF],
            mod_rep: [0xFFFF, 0xFFFF],
            port_a_podr: 0x00,
            broken: false,
        };
        s.init();
        s
    }

    pub const fn break_down(&mut self) {
        self.broken = true;
    }

    pub const fn repair(&mut self) {
        self.broken = false;
    }

    #[must_use]
    pub const fn rx(&self) -> RxMessage {
        RxMessage::new(
            self.rx_data,
            Ack::new()
                .with_err(self.err)
                .with_msg_id(self.last_msg_id.get() & 0x0F),
        )
    }

    pub fn send(&mut self, tx: &[TxMessage]) {
        if self.broken {
            return;
        }
        self.ecat_recv(&tx[self.idx]);
    }

    pub fn init(&mut self) {
        self.fpga.init();
        unsafe {
            _ = self.clear(&[]);
        }
    }

    pub fn update(&mut self) {
        self.update_with_sys_time(DcSysTime::now());
    }

    pub fn update_with_sys_time(&mut self, sys_time: DcSysTime) {
        self.fpga.update_with_sys_time(sys_time);
        self.read_fpga_state();
        self.dc_sys_time = sys_time;
    }

    #[must_use]
    pub const fn should_update(&self) -> bool {
        self.reads_fpga_state
    }

    pub fn set_last_msg_id(&mut self, msg_id: MsgId) {
        self.last_msg_id = msg_id;
    }
}

impl CPUEmulator {
    #[must_use]
    pub(crate) const fn cast<T>(data: &[u8]) -> T {
        unsafe { (data.as_ptr() as *const T).read_unaligned() }
    }

    #[must_use]
    const fn get_addr(select: u8, addr: u16) -> u16 {
        ((select as u16 & 0x0003) << 14) | (addr & 0x3FFF)
    }

    #[must_use]
    pub(crate) fn bram_read(&self, select: u8, addr: u16) -> u16 {
        let addr = Self::get_addr(select, addr);
        self.fpga.read(addr)
    }

    pub(crate) fn bram_write(&mut self, select: u8, addr: u16, data: u16) {
        let addr = Self::get_addr(select, addr);
        self.fpga.write(addr, data)
    }

    pub(crate) fn bram_cpy(
        &mut self,
        select: u8,
        addr_base: u16,
        src: *const u16,
        size: usize,
    ) -> *const u16 {
        let mut addr = Self::get_addr(select, addr_base);
        let mut src = src;
        (0..size).for_each(|_| unsafe {
            self.fpga.write(addr, src.read());
            addr = addr.wrapping_add(1);
            src = src.add(1);
        });
        src
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
        if self.reads_fpga_state {
            self.rx_data = FPGA_STATE_READS_FPGA_STATE_ENABLED
                | self.bram_read(BRAM_SELECT_CONTROLLER, ADDR_FPGA_STATE) as u8;
        } else {
            self.rx_data &= !FPGA_STATE_READS_FPGA_STATE_ENABLED;
        }
    }

    #[must_use]
    fn handle_payload(&mut self, data: &[u8]) -> u8 {
        unsafe {
            match data[0] {
                TAG_NOP => NO_ERR,
                TAG_CLEAR => self.clear(data),
                TAG_SYNC => self.synchronize(data),
                TAG_FIRM_INFO => self.firm_info(data),
                TAG_MODULATION => self.write_mod(data),
                TAG_MODULATION_CHANGE_SEGMENT => self.change_mod_segment(data),
                TAG_SILENCER => self.config_silencer(data),
                TAG_GAIN => self.write_gain(data),
                TAG_GAIN_CHANGE_SEGMENT => self.change_gain_segment(data),
                TAG_GAIN_STM_CHANGE_SEGMENT => self.change_gain_stm_segment(data),
                TAG_FOCI_STM => self.write_foci_stm(data),
                TAG_FOCI_STM_CHANGE_SEGMENT => self.change_foci_stm_segment(data),
                TAG_GAIN_STM => self.write_gain_stm(data),
                TAG_FORCE_FAN => self.configure_force_fan(data),
                TAG_READS_FPGA_STATE => self.configure_reads_fpga_state(data),
                TAG_CONFIG_PULSE_WIDTH_ENCODER => self.config_pwe(data),
                TAG_DEBUG => self.config_gpio_output(data),
                TAG_EMULATE_GPIO_IN => self.emulate_gpio_in(data),
                TAG_CPU_GPIO_OUT => self.cpu_gpio_out(data),
                TAG_PHASE_CORRECTION => self.phase_corr(data),
                _ => ERR_NOT_SUPPORTED_TAG,
            }
        }
    }

    fn ecat_recv(&mut self, data: *const TxMessage) {
        let data: &[u8] = unsafe { std::slice::from_raw_parts(data as _, EC_OUTPUT_FRAME_SIZE) };

        let header = unsafe { &*(data.as_ptr() as *const Header) };

        if self.last_msg_id == header.msg_id {
            return;
        }
        self.last_msg_id = header.msg_id;

        self.read_fpga_state();

        if header.msg_id > MsgId::MAX {
            self.err = ERR_INVALID_MSG_ID;
            return;
        }

        self.err = self.handle_payload(&data[std::mem::size_of::<Header>()..]);
        if self.err != NO_ERR {
            return;
        }

        if header.slot_2_offset != 0 {
            self.err = self.handle_payload(
                &data[std::mem::size_of::<Header>() + header.slot_2_offset as usize..],
            );
            if self.err != NO_ERR {
                return;
            }
        }

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_CTL_FLAG,
            self.fpga_flags_internal,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    #[test]
    fn cpu_idx() {
        let mut rng = rand::rng();
        let idx: u16 = rng.random();
        let cpu = CPUEmulator::new(idx as _, 249);
        assert_eq!(idx as usize, cpu.idx());
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
        cpu.update_with_sys_time(sys_time);
        assert_eq!(sys_time, cpu.dc_sys_time());
    }

    #[test]
    fn should_update() {
        let mut cpu = CPUEmulator::new(0, 249);
        assert!(!cpu.should_update());

        cpu.reads_fpga_state = true;
        assert!(cpu.should_update());
    }
}
