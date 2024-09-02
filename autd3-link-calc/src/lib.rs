use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    link::{Link, LinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;

pub struct Calc {
    is_open: bool,
    cpus: Vec<CPUEmulator>,
    timeout: std::time::Duration,
}

#[derive(Builder)]
pub struct CalcBuilder {
    #[get]
    #[set]
    timeout: std::time::Duration,
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for CalcBuilder {
    type L = Calc;

    async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError> {
        Ok(Calc {
            is_open: true,
            cpus: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
                .collect(),
            timeout: self.timeout,
        })
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for Calc {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        self.is_open = false;
        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.send(tx);
        });

        Ok(true)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        self.cpus.iter_mut().for_each(|cpu| {
            cpu.update();
            rx[cpu.idx()] = cpu.rx();
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

impl Calc {
    pub const fn builder() -> CalcBuilder {
        CalcBuilder {
            timeout: std::time::Duration::ZERO,
        }
    }
}
