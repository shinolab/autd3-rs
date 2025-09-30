use std::time::Duration;

use core_affinity::CoreId;
use ethercrab::{MainDeviceConfig, Timeouts, subdevice_group::DcConfiguration};
use thread_priority::ThreadBuilder;

use super::EtherCrabOption;
use crate::error::EtherCrabError;

/// A full option for [`EtherCrab`]. See also [`EtherCrabOption`] for default settings.
///
/// [`EtherCrab`]: crate::EtherCrab
#[derive(Clone, Debug)]
pub struct EtherCrabOptionFull {
    /// The network interface name. If `None`, the network interface will be automatically selected to which the AUTD3 device is connected.
    pub ifname: Option<String>,
    /// The size of the send queue buffer.
    pub buf_size: usize,
    /// See [`Timeouts`].
    pub timeouts: Timeouts,
    /// See [`MainDeviceConfig`].
    pub main_device_config: MainDeviceConfig,
    /// See [`DcConfiguration`].
    pub dc_configuration: DcConfiguration,
    /// The interval to check the state.
    pub state_check_period: Duration,
    /// The synchronization tolerance.
    pub sync_tolerance: Duration,
    /// The synchronization timeout.
    pub sync_timeout: Duration,
    /// The [`ThreadBuilder`] for the TX/RX thread.
    pub tx_rx_thread_builder: ThreadBuilder,
    /// The CPU affinity for the TX/RX thread.
    pub tx_rx_thread_affinity: Option<CoreId>,
    /// The [`ThreadBuilder`] for the main thread.
    pub main_thread_builder: ThreadBuilder,
    /// The CPU affinity for the main thread.
    pub main_thread_affinity: Option<CoreId>,
}

impl Default for EtherCrabOptionFull {
    fn default() -> Self {
        EtherCrabOption::default().into()
    }
}

impl EtherCrabOptionFull {
    pub(crate) async fn ifname(&self) -> Result<String, EtherCrabError> {
        match self.ifname.as_ref() {
            Some(ifname) => Ok(ifname.clone()),
            None => {
                tracing::info!(target: "autd3-link-ethercrab", "No interface name is specified. Looking for AUTD device...");
                let ifname = crate::inner::utils::lookup_autd().await?;
                tracing::info!(target: "autd3-link-ethercrab", "Found EtherCAT device on {:?}", ifname);
                Ok(ifname)
            }
        }
    }
}
