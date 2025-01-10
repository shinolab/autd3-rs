use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxMessage},
    link::{Link, LinkBuilder},
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

impl LinkBuilder for NopBuilder {
    type L = Nop;

    fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDDriverError> {
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

impl Link for Nop {
    fn close(&mut self) -> Result<(), AUTDDriverError> {
        self.is_open = false;
        Ok(())
    }

    fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError> {
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(tx);
        });

        Ok(true)
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError> {
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

#[cfg(feature = "async")]
use autd3_driver::link::{AsyncLink, AsyncLinkBuilder};

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl AsyncLinkBuilder for NopBuilder {
    type L = Nop;

    async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDDriverError> {
        <Self as LinkBuilder>::open(self, geometry)
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl AsyncLink for Nop {
    async fn close(&mut self) -> Result<(), AUTDDriverError> {
        <Self as Link>::close(self)
    }

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError> {
        <Self as Link>::send(self, tx)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError> {
        <Self as Link>::receive(self, rx)
    }

    fn is_open(&self) -> bool {
        <Self as Link>::is_open(self)
    }
}

impl Nop {
    /// Create a new [`NopBuilder`].
    pub const fn builder() -> NopBuilder {
        NopBuilder {}
    }
}
