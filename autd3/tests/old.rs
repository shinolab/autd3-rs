#![allow(clippy::missing_transmute_annotations)]

use autd3_core::{
    ethercat::DcSysTime,
    link::{Link, TxBufferPoolSync},
};
use autd3_firmware_emulator_v11::CPUEmulator;

#[derive(Default)]
pub struct OldFirmwareEmulator {
    is_open: bool,
    cpus: Vec<CPUEmulator>,
    buffer_pool: TxBufferPoolSync,
}

impl Link for OldFirmwareEmulator {
    fn open(
        &mut self,
        geometry: &autd3::prelude::Geometry,
    ) -> Result<(), autd3_core::link::LinkError> {
        self.is_open = true;
        self.cpus = geometry
            .iter()
            .enumerate()
            .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
            .collect();
        self.buffer_pool.init(geometry);
        Ok(())
    }

    fn close(&mut self) -> Result<(), autd3_core::link::LinkError> {
        self.is_open = false;
        Ok(())
    }

    fn alloc_tx_buffer(
        &mut self,
    ) -> Result<Vec<autd3_core::link::TxMessage>, autd3_core::link::LinkError> {
        Ok(self.buffer_pool.borrow())
    }

    fn send(
        &mut self,
        tx: Vec<autd3_core::link::TxMessage>,
    ) -> Result<(), autd3_core::link::LinkError> {
        {
            let tx: &[autd3_core::link::TxMessage] = &tx;
            self.cpus.iter_mut().for_each(|cpu| {
                cpu.send(unsafe { std::mem::transmute(tx) });
            });
        }
        self.buffer_pool.return_buffer(tx);

        Ok(())
    }

    fn receive(
        &mut self,
        rx: &mut [autd3_core::link::RxMessage],
    ) -> Result<(), autd3_core::link::LinkError> {
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.update_with_sys_time(unsafe { std::mem::transmute(DcSysTime::ZERO) });
            rx[cpu.idx()] = unsafe { std::mem::transmute(cpu.rx()) };
        });

        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

#[test]
fn unsupported_firmware() {
    use autd3::{Controller, core::devices::AUTD3, driver::error::AUTDDriverError};

    let r = Controller::open([AUTD3::default()], OldFirmwareEmulator::default());
    match r {
        Err(AUTDDriverError::UnsupportedFirmware) => {}
        _ => panic!("Expected UnsupportedFirmware error"),
    }
}
