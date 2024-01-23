use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    error::AUTDInternalError,
    geometry::Geometry,
    link::{Link, LinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

/// Link to do nothing
pub struct Nop {
    is_open: bool,
    cpus: Vec<CPUEmulator>,
    timeout: std::time::Duration,
}

pub struct NopBuilder {
    timeout: std::time::Duration,
}

impl NopBuilder {
    #[allow(clippy::needless_update)]
    pub fn with_timeout(self, timeout: std::time::Duration) -> Self {
        Self { timeout, ..self }
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for NopBuilder {
    type L = Nop;

    async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError> {
        Ok(Nop {
            is_open: true,
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
                .collect(),
            timeout: self.timeout,
        })
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for Nop {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        self.is_open = false;
        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        if !self.is_open {
            return Err(AUTDInternalError::LinkClosed);
        }

        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(tx);
        });

        Ok(true)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        if !self.is_open {
            return Err(AUTDInternalError::LinkClosed);
        }

        self.cpus.iter_mut().for_each(|cpu| {
            rx[cpu.idx()].data = cpu.rx_data();
            rx[cpu.idx()].ack = cpu.ack();
        });

        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn timeout(&self) -> std::time::Duration {
        self.timeout
    }
}

impl Nop {
    pub const fn builder() -> NopBuilder {
        NopBuilder {
            timeout: std::time::Duration::ZERO,
        }
    }
}
