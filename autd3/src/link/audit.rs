use std::time::Duration;

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    link::{Link, LinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut, Builder)]
pub struct Audit {
    is_open: bool,
    #[get]
    timeout: Duration,
    #[get]
    last_timeout: Duration,
    #[get]
    last_parallel_threshold: usize,
    #[deref]
    #[deref_mut]
    cpus: Vec<CPUEmulator>,
    down: bool,
    broken: bool,
}

#[derive(Builder)]
pub struct AuditBuilder {
    #[get]
    #[set]
    timeout: Duration,
    #[get]
    #[set]
    initial_msg_id: Option<u8>,
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
            last_parallel_threshold: 4,
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| {
                    let mut cpu = CPUEmulator::new(i, dev.num_transducers());
                    if let Some(msg_id) = self.initial_msg_id {
                        cpu.set_last_msg_id(msg_id);
                    }
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
            initial_msg_id: None,
        }
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

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for Audit {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        self.is_open = false;
        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
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

    fn trace(
        &mut self,
        _: &TxDatagram,
        _: &mut [RxMessage],
        timeout: Duration,
        parallel_threshold: usize,
    ) {
        self.last_timeout = timeout;
        self.last_parallel_threshold = parallel_threshold;
    }
}
