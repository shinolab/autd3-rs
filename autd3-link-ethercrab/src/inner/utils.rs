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
    tracing::debug!(target: "autd3-link-ethercrab", "Found {} network interfaces.", devices.len());
    for interface in devices.into_iter() {
        tracing::debug!(target: "autd3-link-ethercrab",
            "Searching AUTD device on {} ({}).",
            interface.name,
            interface.desc.as_deref().unwrap_or("No description")
        );

        let pdu_storage: Box<PduStorage<MAX_FRAMES, MAX_PDU_DATA>> = Box::new(PduStorage::new());
        let pdu_storage = Box::leak(pdu_storage);
        let (tx, rx, pdu_loop) = match pdu_storage.try_split() {
            Ok((tx, rx, pdu_loop)) => (tx, rx, pdu_loop),
            Err(e) => {
                tracing::error!(target: "autd3-link-ethercrab", "Failed to split PDU storage: {:?}", e);
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
                tracing::trace!(target: "autd3-link-ethercrab", "Failed to start TX/RX task: {}", e);
                Err(EtherCrabError::from(e))
            }
        });

        let found = match executor::block_on(
            main_device.init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now),
        ) {
            Ok(group) => {
                tracing::trace!(target: "autd3-link-ethercrab", "Find EtherCAT device on {}", interface.name);
                !group.is_empty()
                    && group
                        .iter(&main_device)
                        .all(|sub_device| sub_device.name() == "AUTD")
            }
            Err(e) => {
                tracing::trace!(target: "autd3-link-ethercrab", "Failed to initialize EtherCAT on {}: {}", interface.name, e);
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
