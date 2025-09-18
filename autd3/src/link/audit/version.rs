use crate::link::AuditOption;

use autd3_core::link::{RxMessage, TxMessage};

pub use autd3_firmware_emulator::CPUEmulator as V12_1;
#[cfg(feature = "link-audit-v10")]
pub use autd3_firmware_emulator_v10::CPUEmulator as V10;
#[cfg(feature = "link-audit-v11")]
pub use autd3_firmware_emulator_v11::CPUEmulator as V11;
#[cfg(feature = "link-audit-v12")]
pub use autd3_firmware_emulator_v12::CPUEmulator as V12;

#[doc(hidden)]
pub trait Emulator: Send {
    fn new(idx: usize, num_transducers: usize, option: AuditOption) -> Self;
    fn send(&mut self, tx: &[TxMessage]);
    fn update(&mut self);
    fn rx(&self) -> RxMessage;
    fn idx(&self) -> usize;
}

impl Emulator for V12_1 {
    fn new(idx: usize, num_transducers: usize, option: AuditOption) -> Self {
        let mut cpu = V12_1::new(idx, num_transducers);
        if let Some(msg_id) = option.initial_msg_id {
            cpu.set_last_msg_id(msg_id);
        }
        if let Some(initial_phase_corr) = option.initial_phase_corr {
            cpu.fpga_mut()
                .set_phase_corr_bram(u16::from_le_bytes([initial_phase_corr, initial_phase_corr]));
        }
        cpu
    }

    fn send(&mut self, tx: &[TxMessage]) {
        V12_1::send(self, tx)
    }

    fn update(&mut self) {
        V12_1::update(self);
    }

    fn rx(&self) -> RxMessage {
        V12_1::rx(self)
    }

    fn idx(&self) -> usize {
        V12_1::idx(self)
    }
}

#[cfg(feature = "link-audit-v12")]
impl Emulator for V12 {
    fn new(idx: usize, num_transducers: usize, _: AuditOption) -> Self {
        V12::new(idx, num_transducers)
    }

    #[allow(clippy::missing_transmute_annotations)]
    fn send(&mut self, tx: &[TxMessage]) {
        V12::send(self, unsafe { std::mem::transmute(tx) })
    }

    fn update(&mut self) {
        V12::update(self);
    }

    fn rx(&self) -> RxMessage {
        unsafe { std::mem::transmute(V12::rx(self)) }
    }

    fn idx(&self) -> usize {
        V12::idx(self)
    }
}

#[cfg(feature = "link-audit-v11")]
impl Emulator for V11 {
    fn new(idx: usize, num_transducers: usize, _: AuditOption) -> Self {
        V11::new(idx, num_transducers)
    }

    #[allow(clippy::missing_transmute_annotations)]
    fn send(&mut self, tx: &[TxMessage]) {
        V11::send(self, unsafe { std::mem::transmute(tx) })
    }

    fn update(&mut self) {
        V11::update(self);
    }

    fn rx(&self) -> RxMessage {
        unsafe { std::mem::transmute(V11::rx(self)) }
    }

    fn idx(&self) -> usize {
        V11::idx(self)
    }
}

#[cfg(feature = "link-audit-v10")]
impl Emulator for V10 {
    fn new(idx: usize, num_transducers: usize, _: AuditOption) -> Self {
        V10::new(idx, num_transducers)
    }

    #[allow(clippy::missing_transmute_annotations)]
    fn send(&mut self, tx: &[TxMessage]) {
        V10::send(self, unsafe { std::mem::transmute(tx) })
    }

    fn update(&mut self) {
        V10::update(self);
    }

    fn rx(&self) -> RxMessage {
        unsafe { std::mem::transmute(V10::rx(self)) }
    }

    fn idx(&self) -> usize {
        V10::idx(self)
    }
}
