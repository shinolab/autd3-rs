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

impl PartialEq for EtherCrabOptionFull {
    fn eq(&self, other: &Self) -> bool {
        let Timeouts {
            state_transition,
            pdu,
            eeprom,
            wait_loop_delay,
            mailbox_echo,
            mailbox_response,
        } = self.timeouts;
        let DcConfiguration {
            start_delay,
            sync0_period,
            sync0_shift,
        } = self.dc_configuration;
        self.ifname == other.ifname
            && self.buf_size == other.buf_size
            && state_transition == other.timeouts.state_transition
            && pdu == other.timeouts.pdu
            && eeprom == other.timeouts.eeprom
            && wait_loop_delay == other.timeouts.wait_loop_delay
            && mailbox_echo == other.timeouts.mailbox_echo
            && mailbox_response == other.timeouts.mailbox_response
            && self.main_device_config == other.main_device_config
            && start_delay == other.dc_configuration.start_delay
            && sync0_period == other.dc_configuration.sync0_period
            && sync0_shift == other.dc_configuration.sync0_shift
            && self.state_check_period == other.state_check_period
            && self.sync_tolerance == other.sync_tolerance
            && self.sync_timeout == other.sync_timeout
            && self.tx_rx_thread_affinity == other.tx_rx_thread_affinity
            && self.main_thread_affinity == other.main_thread_affinity
    }
}
impl Eq for EtherCrabOptionFull {}

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
