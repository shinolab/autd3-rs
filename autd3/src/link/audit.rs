use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxMessage},
    link::{Link, LinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

use derive_more::{Deref, DerefMut};

/// A [`Link`] for testing.
#[derive(Deref, DerefMut, Builder)]
pub struct Audit {
    is_open: bool,
    #[deref]
    #[deref_mut]
    cpus: Vec<CPUEmulator>,
    down: bool,
    broken: bool,
    #[get]
    /// The last parallel threshold.
    last_parallel_threshold: Option<usize>,
    #[get]
    /// The last timeout.
    last_timeout: Option<std::time::Duration>,
}

/// A builder for [`Audit`].
#[derive(Builder)]
pub struct AuditBuilder {
    #[get]
    #[set]
    /// The initial message ID. The default value is `None`.
    initial_msg_id: Option<u8>,
    #[get]
    #[set]
    /// The initial phase correction. The default value is `None`.
    initial_phase_corr: Option<u8>,
    #[get]
    #[set]
    /// The initial state of the link. The default value is `false`.
    down: bool,
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for AuditBuilder {
    type L = Audit;

    async fn open(
        self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<Self::L, AUTDDriverError> {
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
    /// Create a new [`AuditBuilder`].
    pub const fn builder() -> AuditBuilder {
        AuditBuilder {
            initial_msg_id: None,
            initial_phase_corr: None,
            down: false,
        }
    }

    /// Set this link to be down.
    ///
    /// After calling this method, [`Link::send`] and [`Link::receive`] will return `false`.
    pub fn down(&mut self) {
        self.down = true;
    }

    /// Set this link to be up.
    ///
    /// This methods is used to recover the link from [`Audit::down`].
    pub fn up(&mut self) {
        self.down = false;
    }

    /// Break down this link.
    ///
    /// After calling this method, [`Link::send`] and [`Link::receive`] will return an error.
    pub fn break_down(&mut self) {
        self.broken = true;
    }

    /// Repair this link.
    ///
    /// This methods is used to recover the link from [`Audit::break_down`].
    pub fn repair(&mut self) {
        self.broken = false;
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for Audit {
    async fn close(&mut self) -> Result<(), AUTDDriverError> {
        self.is_open = false;
        Ok(())
    }

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError> {
        if self.broken {
            return Err(AUTDDriverError::LinkError("broken".to_owned()));
        }

        if self.down {
            return Ok(false);
        }

        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(tx);
        });

        Ok(true)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError> {
        if self.broken {
            return Err(AUTDDriverError::LinkError("broken".to_owned()));
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
