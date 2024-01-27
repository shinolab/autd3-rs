use async_channel::{SendError, Sender};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

use autd3_driver::{
    cpu::{RxMessage, TxDatagram},
    error::AUTDInternalError,
    link::Link,
    osal_timer::Timer,
};

use crate::local::{
    builder::SOEMBuilder, error::SOEMError, iomap::IOMap, sleep::SoemCallback, soem_bindings::*,
};

/// Link using [SOEM](https://github.com/OpenEtherCATsociety/SOEM)
pub struct SOEM {
    pub(crate) ecatth_handle: Option<JoinHandle<Result<(), SOEMError>>>,
    pub(crate) timer_handle: Option<Box<Timer<SoemCallback>>>,
    pub(crate) ecat_check_th: Option<JoinHandle<()>>,
    pub(crate) timeout: std::time::Duration,
    pub(crate) sender: Sender<TxDatagram>,
    pub(crate) is_open: Arc<AtomicBool>,
    pub(crate) ec_sync0_cycle: std::time::Duration,
    pub(crate) io_map: Arc<Mutex<IOMap>>,
}

impl SOEM {
    pub fn builder() -> SOEMBuilder {
        SOEMBuilder::default()
    }

    pub fn num_devices() -> usize {
        unsafe { ec_slavecount as usize }
    }

    pub async fn clear_iomap(
        &mut self,
    ) -> Result<(), std::sync::PoisonError<std::sync::MutexGuard<'_, IOMap>>> {
        while !self.sender.is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        self.io_map.lock()?.clear();
        Ok(())
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for SOEM {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        if !self.is_open() {
            return Ok(());
        }
        self.is_open.store(false, Ordering::Release);

        while !self.sender.is_empty() {
            tokio::time::sleep(self.ec_sync0_cycle).await;
        }

        if let Some(timer) = self.ecatth_handle.take() {
            let _ = timer.join();
        }
        if let Some(timer) = self.timer_handle.take() {
            timer.close()?;
        }
        if let Some(th) = self.ecat_check_th.take() {
            let _ = th.join();
        }

        unsafe {
            let cyc_time = *(ecx_context.userdata as *mut u32);
            (1..=ec_slavecount as u16).for_each(|i| {
                ec_dcsync0(i, 0, cyc_time, 0);
            });

            ec_slave[0].state = ec_state_EC_STATE_INIT as _;
            ec_writestate(0);

            ec_close();

            let _ = Box::from_raw(ecx_context.userdata as *mut std::time::Duration);
        }

        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        if !self.is_open() {
            return Err(AUTDInternalError::LinkClosed);
        }

        match self.sender.send(tx.clone()).await {
            Err(SendError(..)) => Err(AUTDInternalError::LinkClosed),
            _ => Ok(true),
        }
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        if !self.is_open() {
            return Err(AUTDInternalError::LinkClosed);
        }
        match self.io_map.lock() {
            Ok(io_map) => unsafe {
                std::ptr::copy_nonoverlapping(io_map.input(), rx.as_mut_ptr(), rx.len());
            },
            Err(_) => return Err(AUTDInternalError::LinkClosed),
        }
        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Acquire)
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }
}
