mod props;
mod sub;

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    geometry::Device,
    link::{Link, LinkBuilder},
};
use autd3_firmware_emulator::CPUEmulator;
use sub::SubDevice;

use derive_more::Deref;

#[derive(Deref)]
pub struct Calc {
    last_geometry_version: usize,
    is_open: bool,
    #[deref]
    sub_devices: Vec<SubDevice>,
    timeout: std::time::Duration,
}

#[derive(Builder)]
pub struct CalcBuilder {
    #[get]
    #[set]
    timeout: std::time::Duration,
}

fn clone_device(dev: &Device) -> Device {
    Device::new(dev.idx(), *dev.rotation(), dev.iter().cloned().collect())
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for CalcBuilder {
    type L = Calc;

    async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError> {
        Ok(Calc {
            last_geometry_version: geometry.version(),
            is_open: true,
            sub_devices: geometry
                .iter()
                .enumerate()
                .map(|(i, dev)| SubDevice {
                    device: clone_device(dev),
                    cpu: CPUEmulator::new(i, dev.num_transducers()),
                })
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
        self.sub_devices.iter_mut().for_each(|sub| {
            sub.cpu.send(tx);
        });

        Ok(true)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        self.sub_devices.iter_mut().for_each(|sub| {
            sub.cpu.update();
            rx[sub.cpu.idx()] = sub.cpu.rx();
        });

        Ok(true)
    }

    async fn update(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.last_geometry_version == geometry.version() {
            return Ok(());
        }
        self.last_geometry_version = geometry.version();
        self.sub_devices
            .iter_mut()
            .zip(geometry.iter())
            .for_each(|(sub, dev)| {
                sub.device = clone_device(dev);
            });
        Ok(())
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
