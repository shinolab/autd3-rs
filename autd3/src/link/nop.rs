use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxMessage},
    link::{AsyncLink, AsyncLinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

/// A [`Link`] that does nothing.
///
/// This link is mainly used for explanation.
pub struct Nop {
    is_open: bool,
    cpus: Vec<CPUEmulator>,
}

/// A builder for [`Nop`].
#[derive(Builder)]
pub struct NopBuilder {}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl AsyncLinkBuilder for NopBuilder {
    type L = Nop;

    async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDDriverError> {
        Ok(Nop {
            is_open: true,
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
                .collect(),
        })
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl AsyncLink for Nop {
    async fn close(&mut self) -> Result<(), AUTDDriverError> {
        self.is_open = false;
        Ok(())
    }

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError> {
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(tx);
        });

        Ok(true)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError> {
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.update();
            rx[cpu.idx()] = cpu.rx();
        });

        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

impl Nop {
    /// Create a new [`NopBuilder`].
    pub const fn builder() -> NopBuilder {
        NopBuilder {}
    }
}
