use autd3_core::{
    derive::DatagramOption,
    geometry::Geometry,
    link::{Link, LinkBuilder, LinkError},
};

use autd3_driver::firmware::cpu::{RxMessage, TxMessage};
use autd3_firmware_emulator::CPUEmulator;

use derive_more::{Deref, DerefMut};
use getset::{CopyGetters, Getters};

#[doc(hidden)]
#[derive(Deref, DerefMut, CopyGetters, Getters)]
pub struct Audit {
    is_open: bool,
    #[deref]
    #[deref_mut]
    cpus: Vec<CPUEmulator>,
    down: bool,
    broken: bool,
    /// The last parallel threshold.
    #[getset(get_copy = "pub")]
    last_parallel_threshold: Option<usize>,
    /// The last timeout.
    #[getset(get_copy = "pub")]
    last_timeout: Option<std::time::Duration>,
}

#[derive(Default)]
#[doc(hidden)]
pub struct AuditOption {
    pub initial_msg_id: Option<u8>,
    pub initial_phase_corr: Option<u8>,
    pub down: bool,
}

#[derive(Default)]
#[doc(hidden)]
pub struct AuditBuilder {
    option: AuditOption,
}

impl LinkBuilder for AuditBuilder {
    type L = Audit;

    fn open(self, geometry: &Geometry) -> Result<Self::L, LinkError> {
        Ok(Audit {
            is_open: true,
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| {
                    let mut cpu = CPUEmulator::new(i, dev.num_transducers());
                    if let Some(msg_id) = self.option.initial_msg_id {
                        cpu.set_last_msg_id(msg_id);
                    }
                    if let Some(initial_phase_corr) = self.option.initial_phase_corr {
                        cpu.fpga_mut()
                            .mem_mut()
                            .phase_corr_bram_mut()
                            .borrow_mut()
                            .fill(u16::from_le_bytes([initial_phase_corr, initial_phase_corr]));
                    }
                    cpu
                })
                .collect(),
            down: self.option.down,
            broken: false,
            last_parallel_threshold: None,
            last_timeout: None,
        })
    }
}

impl Audit {
    pub fn builder(option: AuditOption) -> AuditBuilder {
        AuditBuilder { option }
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

impl Link for Audit {
    fn close(&mut self) -> Result<(), LinkError> {
        self.is_open = false;
        Ok(())
    }

    fn send(&mut self, tx: &[TxMessage]) -> Result<bool, LinkError> {
        if self.broken {
            return Err(LinkError::new("broken".to_owned()));
        }

        if self.down {
            return Ok(false);
        }

        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(tx);
        });

        Ok(true)
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
        if self.broken {
            return Err(LinkError::new("broken".to_owned()));
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

    fn trace(&mut self, option: &DatagramOption) {
        self.last_timeout = Some(option.timeout);
        self.last_parallel_threshold = Some(option.parallel_threshold);
    }
}

#[cfg(feature = "async")]
use autd3_core::link::{AsyncLink, AsyncLinkBuilder};

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncLinkBuilder for AuditBuilder {
    type L = Audit;

    async fn open(self, geometry: &Geometry) -> Result<Self::L, LinkError> {
        <Self as LinkBuilder>::open(self, geometry)
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg_attr(feature = "async-trait", autd3_core::async_trait)]
impl AsyncLink for Audit {
    async fn close(&mut self) -> Result<(), LinkError> {
        <Self as Link>::close(self)
    }

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, LinkError> {
        <Self as Link>::send(self, tx)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
        <Self as Link>::receive(self, rx)
    }

    fn is_open(&self) -> bool {
        <Self as Link>::is_open(self)
    }

    fn trace(&mut self, option: &DatagramOption) {
        <Self as Link>::trace(self, option)
    }
}
