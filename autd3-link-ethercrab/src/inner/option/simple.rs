use std::time::Duration;

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
    /// The interval to check the state. The default is 100ms.
    pub state_check_period: Duration,
    /// The period of the sync0 signal. The default is 2ms.
    pub sync0_period: Duration,
    /// The synchronization tolerance. The default is 1us.
    pub sync_tolerance: Duration,
    /// The synchronization timeout. The default is 10s.
    pub sync_timeout: Duration,
}

impl Default for EtherCrabOption {
    fn default() -> Self {
        Self {
            ifname: None,
            state_check_period: Duration::from_millis(100),
            sync0_period: Duration::from_millis(2),
            sync_tolerance: Duration::from_micros(1),
            sync_timeout: Duration::from_secs(10),
        }
    }
}

impl From<EtherCrabOption> for EtherCrabOptionFull {
    fn from(opt: EtherCrabOption) -> Self {
        Self {
            ifname: opt.ifname,
            buf_size: unsafe { std::num::NonZeroUsize::new_unchecked(16) },
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
            #[cfg(all(not(feature = "tokio"), target_os = "windows"))]
            main_thread_builder: ThreadBuilder::default().name("main-thread").priority(
                ThreadPriority::Os(thread_priority::ThreadPriorityOsValue::from(
                    thread_priority::WinAPIThreadPriority::TimeCritical,
                )),
            ),
            #[cfg(all(not(feature = "tokio"), not(target_os = "windows")))]
            main_thread_builder: ThreadBuilder::default()
                .name("main-thread")
                .priority(
                    thread_priority::ThreadPriorityValue::try_from(99)
                        .map_or_else(|_| ThreadPriority::Max, ThreadPriority::Crossplatform),
                )
                .policy(thread_priority::ThreadSchedulePolicy::Realtime(
                    thread_priority::RealtimeThreadSchedulePolicy::Fifo,
                )),
            #[cfg(all(not(feature = "tokio"), feature = "core_affinity"))]
            main_thread_affinity: None,
            #[cfg(feature = "core_affinity")]
            tx_rx_thread_affinity: None,
        }
    }
}
