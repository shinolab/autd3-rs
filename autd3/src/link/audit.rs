use autd3_core::{
    ethercat::DcSysTime,
    geometry::Geometry,
    link::{Link, LinkError, MsgId, RxMessage, TxBufferPoolSync, TxMessage},
};
use autd3_firmware_emulator::CPUEmulator;

#[derive(Default, Clone, Copy)]
#[doc(hidden)]
pub struct AuditOption {
    pub initial_msg_id: Option<MsgId>,
    pub broken: bool,
}

#[doc(hidden)]
pub struct Audit {
    option: AuditOption,
    is_open: bool,
    cpus: Vec<CPUEmulator>,
    broken: bool,
    buffer_pool: TxBufferPoolSync,
}

impl std::ops::Deref for Audit {
    type Target = [CPUEmulator];

    fn deref(&self) -> &Self::Target {
        &self.cpus
    }
}

impl std::ops::DerefMut for Audit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cpus
    }
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
            cpu.update_with_sys_time(DcSysTime::ZERO);
            rx[cpu.idx()] = cpu.rx();
        });

        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

impl autd3_core::link::AsyncLink for Audit {}
