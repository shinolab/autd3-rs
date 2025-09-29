use std::time::Duration;

use core_affinity::CoreId;
use ethercrab::{MainDeviceConfig, Timeouts, subdevice_group::DcConfiguration};
use thread_priority::ThreadBuilder;

use super::EtherCrabOption;
use crate::EtherCrabError;

#[derive(Clone, Debug)]
pub struct EtherCrabOptionFull {
    pub ifname: Option<String>,
    pub buf_size: usize,
    pub timeouts: Timeouts,
    pub main_device_config: MainDeviceConfig,
    pub dc_configuration: DcConfiguration,
    pub state_check_period: Duration,
    pub sync_tolerance: Duration,
    pub sync_timeout: Duration,
    pub tx_rx_thread_builder: ThreadBuilder,
    pub main_thread_builder: ThreadBuilder,
    pub tx_rx_thread_core: Option<CoreId>,
    pub main_thread_core: Option<CoreId>,
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
