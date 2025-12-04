use std::sync::{Arc, atomic::AtomicBool};

use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, Timeouts, std::ethercat_now};

use crate::{
    error::EtherCrabError,
    inner::handler::{MAX_FRAMES, MAX_PDU_DATA, MAX_SUBDEVICES, PDI_LEN},
    log,
};

pub(crate) struct PduStorageWrapper {
    pdu_storage: *mut PduStorage<MAX_FRAMES, MAX_PDU_DATA>,
}

unsafe impl Send for PduStorageWrapper {}

impl PduStorageWrapper {
    pub fn new() -> Self {
        let pdu_storage: Box<PduStorage<MAX_FRAMES, MAX_PDU_DATA>> = Box::new(PduStorage::new());
        Self {
            pdu_storage: Box::into_raw(pdu_storage),
        }
    }

    pub fn try_split(
        &self,
    ) -> Result<
        (
            ethercrab::PduTx<'static>,
            ethercrab::PduRx<'static>,
            ethercrab::PduLoop<'static>,
        ),
        (),
    > {
        unsafe { (&*self.pdu_storage).try_split() }
    }

    pub fn release(&mut self) {
        if self.pdu_storage.is_null() {
            return;
        }
        let _pdu_storage = unsafe { Box::from_raw(self.pdu_storage) };
        self.pdu_storage = std::ptr::null_mut();
    }
}

impl Drop for PduStorageWrapper {
    fn drop(&mut self) {
        self.release();
    }
}

pub async fn lookup_autd() -> Result<String, EtherCrabError> {
    let devices = pcap::Device::list()?;

    log::debug!("Found {} network interfaces.", devices.len());
    for interface in devices.into_iter() {
        log::debug!(
            "Searching AUTD device on {} ({}).",
            interface.name,
            interface.desc.as_deref().unwrap_or("No description")
        );

        let pdu_storage = PduStorageWrapper::new();
        let (tx, rx, pdu_loop) = match pdu_storage.try_split() {
            Ok((tx, rx, pdu_loop)) => (tx, rx, pdu_loop),
            Err(_e) => {
                log::error!("Failed to split PDU storage: {:?}", _e);
                continue;
            }
        };

        let main_device =
            MainDevice::new(pdu_loop, Timeouts::default(), MainDeviceConfig::default());

        let running = Arc::new(AtomicBool::new(true));

        let device = interface.name.to_string();
        #[cfg(target_os = "windows")]
        let th = std::thread::spawn({
            let running = Arc::clone(&running);
            move || crate::inner::windows::tx_rx_task_blocking(&device, tx, rx, running)
        });

        #[cfg(all(not(target_os = "windows"), not(feature = "tokio")))]
        let th = std::thread::spawn(move || match ethercrab::std::tx_rx_task(&device, tx, rx) {
            Ok(fut) => super::executor::block_on(fut)
                .map(|_| ())
                .map_err(EtherCrabError::from),
            Err(e) => {
                log::trace!("Failed to start TX/RX task: {}", e);
                Err(EtherCrabError::from(e))
            }
        });
        #[cfg(all(not(target_os = "windows"), feature = "tokio"))]
        let th = tokio::task::spawn({
            async move {
                match ethercrab::std::tx_rx_task(&device, tx, rx) {
                    Ok(fut) => fut.await.map(|_| ()).map_err(EtherCrabError::from),
                    Err(e) => {
                        log::trace!("Failed to start TX/RX task: {}", e);
                        Err(EtherCrabError::from(e))
                    }
                }
            }
        });

        let found = match main_device
            .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
            .await
        {
            Ok(group) => {
                log::trace!("Find EtherCAT device on {}", interface.name);
                !group.is_empty()
                    && group
                        .iter(&main_device)
                        .all(|sub_device| sub_device.name() == "AUTD")
            }
            Err(_e) => {
                log::trace!(
                    "Failed to initialize EtherCAT on {}: {}",
                    interface.name,
                    _e
                );
                false
            }
        };

        running.store(false, std::sync::atomic::Ordering::Relaxed);

        #[cfg(target_os = "windows")]
        {
            th.thread().unpark();
            let _ = th.join();
        }
        #[cfg(all(not(target_os = "windows"), not(feature = "tokio")))]
        {
            th.thread().unpark();
            unsafe { main_device.release_all() };
            let _ = th.join();
        }
        #[cfg(all(not(target_os = "windows"), feature = "tokio"))]
        {
            unsafe { main_device.release_all() };
            let _ = th.await;
        }

        if found {
            return Ok(interface.name);
        }
    }
    Err(EtherCrabError::DeviceNotFound)
}
