/*
 * File: audit.rs
 * Project: link
 * Created Date: 14/09/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 19/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    error::AUTDInternalError,
    link::{Link, LinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

/// Link for test
pub struct Audit {
    is_open: bool,
    timeout: Duration,
    last_timeout: Duration,
    cpus: Vec<CPUEmulator>,
    down: bool,
    broken: bool,
}

pub struct AuditBuilder {
    timeout: Duration,
}

impl AuditBuilder {
    /// set timeout
    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self { timeout }
    }
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
            last_timeout: Duration::ZERO,
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| {
                    let mut cpu = CPUEmulator::new(i, dev.num_transducers());
                    cpu.init();
                    cpu
                })
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

    pub fn last_timeout(&self) -> Duration {
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
            rx[cpu.idx()].data = cpu.rx_data();
            rx[cpu.idx()].ack = cpu.ack();
        });

        Ok(true)
    }

    async fn send_receive(
        &mut self,
        tx: &TxDatagram,
        rx: &mut [RxMessage],
        timeout: Option<Duration>,
    ) -> Result<bool, AUTDInternalError> {
        let timeout = timeout.unwrap_or(self.timeout);
        self.last_timeout = timeout;
        if !self.send(tx).await? {
            return Ok(false);
        }
        if timeout.is_zero() {
            return self.receive(rx).await;
        }
        self.wait_msg_processed(tx, rx, timeout).await
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }
}
