use std::time::Duration;

use core_affinity::CoreId;
use ethercrab::{MainDeviceConfig, RetryBehaviour, Timeouts, subdevice_group::DcConfiguration};
use thread_priority::{ThreadBuilder, ThreadPriority};

use super::EtherCrabOptionFull;

/// A option for [`EtherCrab`].
///
/// [`EtherCrab`]: crate::EtherCrab
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtherCrabOption {
    /// The network interface name. If `None`, the network interface will be automatically selected to which the AUTD3 device is connected. The default is `None`.
    pub ifname: Option<String>,
    /// The size of the send queue buffer. The default is 16.
    pub buf_size: usize,
    /// The interval to check the state. The default is 100ms.
    pub state_check_period: Duration,
    /// The period of the sync0 signal. The default is 2ms.
    pub sync0_period: Duration,
    /// The synchronization tolerance. The default is 1us.
    pub sync_tolerance: Duration,
    /// The synchronization timeout. The default is 10s.
    pub sync_timeout: Duration,
    /// The [`ThreadBuilder`] for the TX/RX thread. This is set to use real-time scheduling with the highest possible priority by default.
    pub tx_rx_thread_builder: ThreadBuilder,
    /// The CPU affinity for the TX/RX thread. The default is `None`, which means no affinity is set.
    pub tx_rx_thread_affinity: Option<CoreId>,
    /// The [`ThreadBuilder`] for the main thread. This is set to use real-time scheduling with the highest possible priority by default.
    pub main_thread_builder: ThreadBuilder,
    /// The CPU affinity for the main thread. The default is `None`, which means no affinity is set.
    pub main_thread_affinity: Option<CoreId>,
}

impl Default for EtherCrabOption {
    fn default() -> Self {
        Self {
            ifname: None,
            buf_size: 16,
            state_check_period: Duration::from_millis(100),
            sync0_period: Duration::from_millis(2),
            sync_tolerance: Duration::from_micros(1),
            sync_timeout: Duration::from_secs(10),
            #[cfg(target_os = "windows")]
            tx_rx_thread_builder: ThreadBuilder::default().name("tx-rx-thread").priority(
                ThreadPriority::Os(thread_priority::ThreadPriorityOsValue::from(
                    thread_priority::WinAPIThreadPriority::TimeCritical,
                )),
            ),
            #[cfg(not(target_os = "windows"))]
            tx_rx_thread_builder: ThreadBuilder::default()
                .name("tx-rx-thread")
                .priority(
                    thread_priority::ThreadPriorityValue::try_from(99)
                        .map_or_else(|_| ThreadPriority::Max, ThreadPriority::Crossplatform),
                )
                .policy(thread_priority::ThreadSchedulePolicy::Realtime(
                    thread_priority::RealtimeThreadSchedulePolicy::Fifo,
                )),
            #[cfg(target_os = "windows")]
            main_thread_builder: ThreadBuilder::default().name("main-thread").priority(
                ThreadPriority::Os(thread_priority::ThreadPriorityOsValue::from(
                    thread_priority::WinAPIThreadPriority::TimeCritical,
                )),
            ),
            #[cfg(not(target_os = "windows"))]
            main_thread_builder: ThreadBuilder::default()
                .name("main-thread")
                .priority(
                    thread_priority::ThreadPriorityValue::try_from(99)
                        .map_or_else(|_| ThreadPriority::Max, ThreadPriority::Crossplatform),
                )
                .policy(thread_priority::ThreadSchedulePolicy::Realtime(
                    thread_priority::RealtimeThreadSchedulePolicy::Fifo,
                )),
            main_thread_affinity: None,
            tx_rx_thread_affinity: None,
        }
    }
}

impl From<EtherCrabOption> for EtherCrabOptionFull {
    fn from(opt: EtherCrabOption) -> Self {
        Self {
            ifname: opt.ifname,
            buf_size: opt.buf_size,
            timeouts: Timeouts {
                state_transition: Duration::from_secs(10),
                pdu: Duration::from_millis(100),
                wait_loop_delay: Duration::ZERO,
                ..Default::default()
            },
            main_device_config: MainDeviceConfig {
                dc_static_sync_iterations: 10000,
                retry_behaviour: RetryBehaviour::None,
            },
            dc_configuration: DcConfiguration {
                start_delay: Duration::from_millis(100),
                sync0_period: opt.sync0_period,
                sync0_shift: Duration::ZERO,
            },
            state_check_period: opt.state_check_period,
            sync_tolerance: opt.sync_tolerance,
            sync_timeout: opt.sync_timeout,
            tx_rx_thread_builder: opt.tx_rx_thread_builder,
            main_thread_builder: opt.main_thread_builder,
            main_thread_affinity: opt.main_thread_affinity,
            tx_rx_thread_affinity: opt.tx_rx_thread_affinity,
        }
    }
}
