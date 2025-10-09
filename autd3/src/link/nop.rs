use autd3_core::{
    ethercat::DcSysTime,
    geometry::Geometry,
    link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage},
};

use autd3_firmware_emulator::CPUEmulator;

/// A [`Link`] that does nothing.
///
/// This link is mainly used for explanation.
#[derive(Default)]
pub struct Nop {
    is_open: bool,
    cpus: Vec<CPUEmulator>,
    buffer_pool: TxBufferPoolSync,
}

impl Nop {
    /// Creates a new [`Nop`].
    pub const fn new() -> Self {
        Self {
            is_open: false,
            cpus: Vec::new(),
            buffer_pool: TxBufferPoolSync::new(),
        }
    }
}

impl Link for Nop {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        self.is_open = true;
        self.cpus = geometry
            .iter()
            .enumerate()
            .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
            .collect();
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
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(&tx);
        });
        self.buffer_pool.return_buffer(tx);
        Ok(())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
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
