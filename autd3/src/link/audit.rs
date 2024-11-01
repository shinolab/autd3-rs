use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxMessage},
    link::{Link, LinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut, Builder)]
pub struct Audit {
    is_open: bool,
    #[deref]
    #[deref_mut]
    cpus: Vec<CPUEmulator>,
    down: bool,
    broken: bool,
    #[get]
    last_parallel_threshold: Option<usize>,
    #[get]
    last_timeout: Option<std::time::Duration>,
}

#[derive(Builder)]
pub struct AuditBuilder {
    #[get]
    #[set]
    initial_msg_id: Option<u8>,
    #[get]
    #[set]
    initial_phase_corr: Option<u8>,
    #[get]
    #[set]
    down: bool,
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
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| {
                    let mut cpu = CPUEmulator::new(i, dev.num_transducers());
                    if let Some(msg_id) = self.initial_msg_id {
                        cpu.set_last_msg_id(msg_id);
                    }
                    if let Some(initial_phase_corr) = self.initial_phase_corr {
                        cpu.fpga_mut()
                            .mem_mut()
                            .phase_corr_bram_mut()
                            .fill(u16::from_le_bytes([initial_phase_corr, initial_phase_corr]));
                    }
                    cpu
                })
                .collect(),
            down: self.down,
            broken: false,
            last_parallel_threshold: None,
            last_timeout: None,
        })
    }
}

impl Audit {
    pub const fn builder() -> AuditBuilder {
        AuditBuilder {
            initial_msg_id: None,
            initial_phase_corr: None,
            down: false,
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

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDInternalError> {
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

    fn trace(&mut self, timeout: Option<std::time::Duration>, parallel_threshold: Option<usize>) {
        self.last_timeout = timeout;
        self.last_parallel_threshold = parallel_threshold;
    }
}
