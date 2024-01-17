/*
 * File: nop.rs
 * Project: link
 * Created Date: 06/10/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    derive::*,
    error::AUTDInternalError,
    geometry::Geometry,
    link::{LinkSync, LinkSyncBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

/// Link to do nothing
#[derive(Link)]
pub struct Nop {
    is_open: bool,
    cpus: Vec<CPUEmulator>,
    timeout: std::time::Duration,
}

pub struct NopBuilder {
    timeout: std::time::Duration,
}

impl NopBuilder {
    pub fn with_timeout(self, timeout: std::time::Duration) -> Self {
        Self { timeout, ..self }
    }
}

impl LinkSyncBuilder for NopBuilder {
    type L = Nop;

    fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError> {
        Ok(Nop {
            is_open: true,
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| {
                    let mut cpu = CPUEmulator::new(i, dev.num_transducers());
                    cpu.init();
                    cpu
                })
                .collect(),
            timeout: self.timeout,
        })
    }
}

impl LinkSync for Nop {
    fn close(&mut self) -> Result<(), AUTDInternalError> {
        self.is_open = false;
        Ok(())
    }

    fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        if !self.is_open {
            return Err(AUTDInternalError::LinkClosed);
        }

        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(tx);
        });

        Ok(true)
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
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
