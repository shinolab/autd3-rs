use autd3_core::{
    derive::*,
    link::{Link, LinkError},
};

use autd3_driver::firmware::cpu::{RxMessage, TxMessage};
use autd3_firmware_emulator::CPUEmulator;

/// A [`Link`] that does nothing.
///
/// This link is mainly used for explanation.
#[derive(Default)]
pub struct Nop {
    is_open: bool,
    cpus: Vec<CPUEmulator>,
}

impl Nop {
    /// Creates a new [`Nop`].
    pub const fn new() -> Self {
        Self {
            is_open: false,
            cpus: Vec::new(),
        }
    }
}

impl Link for Nop {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        self.is_open = true;
        self.cpus = geometry
            .iter()
            .enumerate()
            .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
            .collect();
        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        self.is_open = false;
        Ok(())
    }

    fn send(&mut self, tx: &[TxMessage]) -> Result<(), LinkError> {
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(tx);
        });
        Ok(())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.update();
            rx[cpu.idx()] = cpu.rx();
        });
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

#[cfg(feature = "async")]
use autd3_core::link::AsyncLink;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncLink for Nop {
    async fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        <Self as Link>::open(self, geometry)
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        <Self as Link>::close(self)
    }

    async fn send(&mut self, tx: &[TxMessage]) -> Result<(), LinkError> {
        <Self as Link>::send(self, tx)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        <Self as Link>::receive(self, rx)
    }

    fn is_open(&self) -> bool {
        <Self as Link>::is_open(self)
    }
}
