use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    link::{Link, LinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

pub struct Audit {
    is_open: bool,
    timeout: Duration,
    last_timeout: Option<Duration>,
    cpus: Vec<CPUEmulator>,
    down: bool,
    broken: bool,
}

#[derive(Builder)]
pub struct AuditBuilder {
    #[getset]
    timeout: Duration,
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for AuditBuilder {
    type L = Audit;

    async fn open(
        self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<Self::L, AUTDInternalError> {
        Ok(Audit {
            is_open: true,
            timeout: self.timeout,
            last_timeout: None,
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
                .collect(),
            down: false,
            broken: false,
        })
    }
}

impl Audit {
    pub const fn builder() -> AuditBuilder {
        AuditBuilder {
            timeout: Duration::ZERO,
        }
    }

    pub const fn last_timeout(&self) -> Option<Duration> {
        self.last_timeout
    }

    pub fn emulators(&self) -> &[CPUEmulator] {
        &self.cpus
    }

    pub fn emulators_mut(&mut self) -> &mut [CPUEmulator] {
        &mut self.cpus
    }

    pub fn down(&mut self) {
        self.down = true;
    }

    pub fn up(&mut self) {
        self.down = false;
    }

    pub fn break_down(&mut self) {
        self.broken = true;
    }

    pub fn repair(&mut self) {
        self.broken = false;
    }
}

impl Deref for Audit {
    type Target = [CPUEmulator];

    fn deref(&self) -> &Self::Target {
        &self.cpus
    }
}

impl DerefMut for Audit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cpus
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for Audit {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        self.is_open = false;
        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        if !self.is_open {
            return Err(AUTDInternalError::LinkClosed);
        }

        if self.broken {
            return Err(AUTDInternalError::LinkError("broken".to_owned()));
        }

        if self.down {
            return Ok(false);
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

        if self.broken {
            return Err(AUTDInternalError::LinkError("broken".to_owned()));
        }

        if self.down {
            return Ok(false);
        }

        self.cpus.iter_mut().for_each(|cpu| {
            cpu.update();
            rx[cpu.idx()] = cpu.rx();
        });

        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }

    fn trace(&mut self, _: &TxDatagram, _: &mut [RxMessage], timeout: Option<Duration>) {
        self.last_timeout = timeout;
    }
}
