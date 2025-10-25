use std::sync::{Arc, atomic::AtomicBool};

use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, Timeouts, std::ethercat_now};

use crate::{
    error::EtherCrabError,
    inner::{
        executor,
        handler::{MAX_FRAMES, MAX_PDU_DATA, MAX_SUBDEVICES, PDI_LEN},
    },
};

pub fn lookup_autd() -> Result<String, EtherCrabError> {
    let devices = pcap::Device::list()?;
    #[cfg(feature = "tracing")]
    tracing::debug!("Found {} network interfaces.", devices.len());
    for interface in devices.into_iter() {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Searching AUTD device on {} ({}).",
            interface.name,
            interface.desc.as_deref().unwrap_or("No description")
        );

        let pdu_storage: Box<PduStorage<MAX_FRAMES, MAX_PDU_DATA>> = Box::new(PduStorage::new());
        let pdu_storage = Box::leak(pdu_storage);
        let (tx, rx, pdu_loop) = match pdu_storage.try_split() {
            Ok((tx, rx, pdu_loop)) => (tx, rx, pdu_loop),
            Err(_e) => {
                #[cfg(feature = "tracing")]
                tracing::error!("Failed to split PDU storage: {:?}", _e);
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

        #[cfg(not(target_os = "windows"))]
        let th = std::thread::spawn(move || match ethercrab::std::tx_rx_task(&device, tx, rx) {
            Ok(fut) => executor::block_on(fut)
                .map(|_| ())
                .map_err(EtherCrabError::from),
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::trace!("Failed to start TX/RX task: {}", e);
                Err(EtherCrabError::from(e))
            }
        });

        let found = match executor::block_on(
            main_device.init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now),
        ) {
            Ok(group) => {
                #[cfg(feature = "tracing")]
                tracing::trace!("Find EtherCAT device on {}", interface.name);
                !group.is_empty()
                    && group
                        .iter(&main_device)
                        .all(|sub_device| sub_device.name() == "AUTD")
            }
            Err(_e) => {
                #[cfg(feature = "tracing")]
                tracing::trace!(
                    "Failed to initialize EtherCAT on {}: {}",
                    interface.name,
                    _e
                );
                false
            }
        };

        running.store(false, std::sync::atomic::Ordering::Relaxed);

        th.thread().unpark();
        #[cfg(target_os = "windows")]
        let _ = th.join();
        #[cfg(not(target_os = "windows"))]
        {
            unsafe { main_device.release_all() };
            let _ = th.join();
        }

        if found {
            return Ok(interface.name);
        }
    }
    Err(EtherCrabError::DeviceNotFound)
}
