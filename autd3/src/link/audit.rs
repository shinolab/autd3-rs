use autd3_core::{
    geometry::Geometry,
    link::{Link, LinkError, MsgId, TxBufferPoolSync},
};

use autd3_driver::firmware::cpu::{RxMessage, TxMessage};
use autd3_firmware_emulator::CPUEmulator;

use derive_more::{Deref, DerefMut};

#[derive(Default)]
#[doc(hidden)]
pub struct AuditOption {
    pub initial_msg_id: Option<MsgId>,
    pub initial_phase_corr: Option<u8>,
    pub broken: bool,
}

#[doc(hidden)]
#[derive(Deref, DerefMut)]
pub struct Audit {
    option: AuditOption,
    is_open: bool,
    #[deref]
    #[deref_mut]
    cpus: Vec<CPUEmulator>,
    broken: bool,
    buffer_pool: TxBufferPoolSync,
}

impl Audit {
    pub const fn new(option: AuditOption) -> Self {
        Self {
            option,
            is_open: false,
            cpus: Vec::new(),
            broken: false,
            buffer_pool: TxBufferPoolSync::new(),
        }
    }

    pub const fn break_down(&mut self) {
        self.broken = true;
    }

    pub const fn repair(&mut self) {
        self.broken = false;
    }
}

impl Link for Audit {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        self.is_open = true;
        self.cpus = geometry
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
            .collect();
        self.broken = self.option.broken;
        self.buffer_pool.init(geometry);
        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        self.is_open = false;
        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        Ok(self.buffer_pool.borrow())
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        if self.broken {
            return Err(LinkError::new("broken"));
        }

        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(&tx);
        });

        self.buffer_pool.return_buffer(tx);

        Ok(())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        if self.broken {
            return Err(LinkError::new("broken"));
        }
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
impl AsyncLink for Audit {
    async fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        <Self as Link>::open(self, geometry)
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        <Self as Link>::close(self)
    }

    async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        <Self as Link>::alloc_tx_buffer(self)
    }

    async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        <Self as Link>::send(self, tx)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        <Self as Link>::receive(self, rx)
    }

    fn is_open(&self) -> bool {
        <Self as Link>::is_open(self)
    }
}
